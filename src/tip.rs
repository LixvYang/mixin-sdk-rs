use ed25519_dalek::Signer;
use sha2::{Digest, Sha256};

use crate::error::Error;

pub const TIP_VERIFY: &str = "TIP:VERIFY:";
pub const TIP_ADDRESS_ADD: &str = "TIP:ADDRESS:ADD:";
pub const TIP_ADDRESS_REMOVE: &str = "TIP:ADDRESS:REMOVE:";
pub const TIP_USER_DEACTIVATE: &str = "TIP:USER:DEACTIVATE:";
pub const TIP_EMERGENCY_CONTACT_CREATE: &str = "TIP:EMERGENCY:CONTACT:CREATE:";
pub const TIP_EMERGENCY_CONTACT_READ: &str = "TIP:EMERGENCY:CONTACT:READ:";
pub const TIP_EMERGENCY_CONTACT_REMOVE: &str = "TIP:EMERGENCY:CONTACT:REMOVE:";
pub const TIP_PHONE_NUMBER_UPDATE: &str = "TIP:PHONE:NUMBER:UPDATE:";
pub const TIP_MULTISIG_REQUEST_SIGN: &str = "TIP:MULTISIG:REQUEST:SIGN:";
pub const TIP_MULTISIG_REQUEST_UNLOCK: &str = "TIP:MULTISIG:REQUEST:UNLOCK:";
pub const TIP_COLLECTIBLE_REQUEST_SIGN: &str = "TIP:COLLECTIBLE:REQUEST:SIGN:";
pub const TIP_COLLECTIBLE_REQUEST_UNLOCK: &str = "TIP:COLLECTIBLE:REQUEST:UNLOCK:";
pub const TIP_TRANSFER_CREATE: &str = "TIP:TRANSFER:CREATE:";
pub const TIP_WITHDRAWAL_CREATE: &str = "TIP:WITHDRAWAL:CREATE:";
pub const TIP_RAW_TRANSACTION_CREATE: &str = "TIP:TRANSACTION:CREATE:";
pub const TIP_OAUTH_APPROVE: &str = "TIP:OAUTH:APPROVE:";
pub const TIP_PROVISIONING_UPDATE: &str = "TIP:PROVISIONING:UPDATE:";
pub const TIP_OWNERSHIP_TRANSFER: &str = "TIP:APP:OWNERSHIP:TRANSFER:";
pub const TIP_SEQUENCER_REGISTER: &str = "SEQUENCER:REGISTER:";

pub fn tip_body(input: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hasher.finalize().to_vec()
}

pub fn tip_body_for_verify(timestamp_nano: i64) -> Vec<u8> {
    tip_body(&format!("{TIP_VERIFY}{timestamp_nano:032}"))
}

pub fn tip_body_for_sequencer_register(user_id: &str, public_key: &str) -> Vec<u8> {
    tip_body(&format!("{TIP_SEQUENCER_REGISTER}{user_id}{public_key}"))
}

pub fn tip_body_for_address_add(
    asset_id: &str,
    destination: &str,
    tag: &str,
    label: &str,
) -> Vec<u8> {
    tip_body(&format!(
        "{TIP_ADDRESS_ADD}{asset_id}{destination}{tag}{label}"
    ))
}

pub fn tip_body_for_transfer(
    asset_id: &str,
    opponent_id: &str,
    amount: &str,
    trace_id: &str,
    memo: &str,
) -> Vec<u8> {
    tip_body(&format!(
        "{TIP_TRANSFER_CREATE}{asset_id}{opponent_id}{amount}{trace_id}{memo}"
    ))
}

pub fn tip_body_for_withdrawal(
    address_id: &str,
    amount: &str,
    fee: &str,
    trace_id: &str,
    memo: &str,
) -> Vec<u8> {
    tip_body(&format!(
        "{TIP_WITHDRAWAL_CREATE}{address_id}{amount}{fee}{trace_id}{memo}"
    ))
}

pub fn tip_body_for_raw_transaction(
    asset_id: &str,
    opponent_key: &str,
    opponent_receivers: &[String],
    opponent_threshold: i64,
    amount: &str,
    trace_id: &str,
    memo: &str,
) -> Vec<u8> {
    let mut body = String::new();
    body.push_str(asset_id);
    body.push_str(opponent_key);
    for receiver in opponent_receivers {
        body.push_str(receiver);
    }
    body.push_str(&opponent_threshold.to_string());
    body.push_str(amount);
    body.push_str(trace_id);
    body.push_str(memo);
    tip_body(&(TIP_RAW_TRANSACTION_CREATE.to_string() + &body))
}

pub fn sign_tip_body(
    body: &[u8],
    spend_private_key: &str,
    _is_spend_private_sum: bool,
) -> Result<String, Error> {
    let key_bytes = hex::decode(spend_private_key)?;
    let seed = match key_bytes.len() {
        32 => key_bytes,
        64 => key_bytes[..32].to_vec(),
        _ => return Err(Error::Input("invalid spend private key length".to_string())),
    };

    let signing_key = ed25519_dalek::SigningKey::from_bytes(
        seed.as_slice()
            .try_into()
            .map_err(|_| Error::Input("invalid spend private key bytes".to_string()))?,
    );
    let signature = signing_key.sign(body);
    Ok(hex::encode(signature.to_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signature, SigningKey, Verifier, VerifyingKey};

    #[test]
    fn test_tip_body_for_sequencer_register() {
        let body = tip_body_for_sequencer_register("user-id", "pub-key");
        let expected = tip_body("SEQUENCER:REGISTER:user-idpub-key");
        assert_eq!(body, expected);
    }

    #[test]
    fn test_sign_tip_body_verifies() {
        let seed = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
        let body = tip_body("TIP:VERIFY:00000000000000000000000000000001");
        let signature_hex = sign_tip_body(&body, seed, false).expect("signature");
        let signature_bytes = hex::decode(signature_hex).expect("sig hex");
        let signature = Signature::from_slice(&signature_bytes).expect("signature");

        let signing_key = SigningKey::from_bytes(&hex::decode(seed).unwrap().try_into().unwrap());
        let verifying_key = VerifyingKey::from(&signing_key);

        verifying_key.verify(&body, &signature).expect("verify");
    }
}
