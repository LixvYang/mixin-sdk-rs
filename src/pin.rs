use aes::Aes256;
use base64::{Engine as _, engine::general_purpose};
use cbc::Encryptor;
use cipher::{BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use curve25519_dalek::edwards::CompressedEdwardsY;
use rand::RngCore;
use sha2::{Digest, Sha512};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{error::Error, safe::SafeUser};

fn private_key_to_curve25519(seed: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha512::new();
    hasher.update(seed);
    let digest = hasher.finalize();

    let mut out = [0u8; 32];
    out.copy_from_slice(&digest[..32]);
    out[0] &= 248;
    out[31] &= 127;
    out[31] |= 64;
    out
}

fn public_key_to_curve25519(public_key: &[u8; 32]) -> Result<[u8; 32], Error> {
    let compressed = CompressedEdwardsY(*public_key);
    let point = compressed
        .decompress()
        .ok_or_else(|| Error::Input("invalid ed25519 public key".to_string()))?;
    Ok(point.to_montgomery().to_bytes())
}

fn shared_key(session_private_key: &[u8; 32], server_public_key: &[u8; 32]) -> Result<[u8; 32], Error> {
    let curve_private = private_key_to_curve25519(session_private_key);
    let curve_public = public_key_to_curve25519(server_public_key)?;

    let shared = x25519_dalek::x25519(curve_private, curve_public);
    Ok(shared)
}

fn unix_seconds() -> Result<u64, Error> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Error::Server(e.to_string()))?
        .as_secs())
}

pub fn encrypt_ed25519_pin(pin_hex: &str, iterator: u64, current: &SafeUser) -> Result<String, Error> {
    encrypt_ed25519_pin_with_time(pin_hex, iterator, current, unix_seconds()?)
}

fn encrypt_ed25519_pin_with_time(
    pin_hex: &str,
    iterator: u64,
    current: &SafeUser,
    now_seconds: u64,
) -> Result<String, Error> {
    if pin_hex.is_empty() {
        return Ok(String::new());
    }

    let private_bytes = hex::decode(&current.session_private_key)?;
    let private: [u8; 32] = private_bytes
        .try_into()
        .map_err(|_| Error::Input("invalid session private key length".to_string()))?;

    if current.server_public_key.is_empty() {
        return Err(Error::Input("missing server public key".to_string()));
    }
    let public_bytes = hex::decode(&current.server_public_key)?;
    let public: [u8; 32] = public_bytes
        .try_into()
        .map_err(|_| Error::Input("invalid server public key length".to_string()))?;

    let key_bytes = shared_key(&private, &public)?;
    let mut pin = hex::decode(pin_hex)?;

    pin.extend_from_slice(&now_seconds.to_le_bytes());
    pin.extend_from_slice(&iterator.to_le_bytes());
    let mut iv = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut iv);

    let msg_len = pin.len();
    let mut buf = pin;
    buf.resize(msg_len + 16, 0);
    let encrypted = Encryptor::<Aes256>::new_from_slices(&key_bytes, &iv)
        .map_err(|e| Error::Server(e.to_string()))?
        .encrypt_padded_mut::<Pkcs7>(&mut buf, msg_len)
        .map_err(|e| Error::Server(e.to_string()))?;

    let mut payload = Vec::with_capacity(iv.len() + encrypted.len());
    payload.extend_from_slice(&iv);
    payload.extend_from_slice(&encrypted);

    Ok(general_purpose::URL_SAFE_NO_PAD.encode(payload))
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use cipher::BlockDecryptMut;

    fn test_user() -> SafeUser {
        let server_seed = [1u8; 32];
        let server_signing_key = ed25519_dalek::SigningKey::from_bytes(&server_seed);
        let server_public_key = hex::encode(server_signing_key.verifying_key().to_bytes());

        SafeUser {
            user_id: "user-id".to_string(),
            session_id: "session-id".to_string(),
            session_private_key: "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f".to_string(),
            server_public_key,
            spend_private_key: "spend-private-key".to_string(),
            is_spend_private_sum: false,
        }
    }

    fn decrypt_payload(key: &[u8; 32], payload: &[u8]) -> Vec<u8> {
        let (iv, ciphertext) = payload.split_at(16);
        let mut buf = ciphertext.to_vec();
        let decryptor = cbc::Decryptor::<Aes256>::new_from_slices(key, iv).unwrap();
        decryptor
            .decrypt_padded_mut::<Pkcs7>(&mut buf)
            .unwrap()
            .to_vec()
    }

    #[test]
    fn test_encrypt_ed25519_pin_roundtrip() {
        let user = test_user();
        let pin_hex = "aabbccddeeff";
        let iterator = 42u64;
        let now_seconds = 1_700_000_000u64;
        let encoded = encrypt_ed25519_pin_with_time(pin_hex, iterator, &user, now_seconds)
            .expect("encrypt");

        let payload = URL_SAFE_NO_PAD.decode(encoded).expect("decode");
        assert!(payload.len() > 16);

        let private_bytes = hex::decode(&user.session_private_key).unwrap();
        let public_bytes = hex::decode(&user.server_public_key).unwrap();
        let key = shared_key(
            &private_bytes.try_into().unwrap(),
            &public_bytes.try_into().unwrap(),
        )
        .unwrap();

        let decrypted = decrypt_payload(&key, &payload);
        let mut expected = hex::decode(pin_hex).unwrap();
        expected.extend_from_slice(&now_seconds.to_le_bytes());
        expected.extend_from_slice(&iterator.to_le_bytes());

        assert_eq!(decrypted, expected);
    }
}
