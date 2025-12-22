use crate::{
    auth,
    error::Error,
    pin::encrypt_ed25519_pin,
    request::{ApiResponse, DEFAULT_API_HOST, DEFAULT_USER_AGENT, HTTP_CLIENT, request},
    tip::{sign_tip_body, tip_body_for_sequencer_register, tip_body_for_verify},
    user::User,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::Signer;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct SafeUser {
    #[serde(rename = "app_id")]
    pub user_id: String,
    #[serde(rename = "session_id")]
    pub session_id: String,
    #[serde(rename = "session_private_key")]
    pub session_private_key: String,
    #[serde(rename = "server_public_key")]
    pub server_public_key: String,
    #[serde(rename = "spend_private_key")]
    pub spend_private_key: String,
    #[serde(skip)]
    pub is_spend_private_sum: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GhostKeys {
    #[serde(rename = "type")]
    pub key_type: String,
    pub mask: String,
    pub keys: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GhostKeyRequest {
    pub receivers: Vec<String>,
    pub index: u32,
    pub hint: String,
}

impl SafeUser {
    pub fn new(
        user_id: String,
        session_id: String,
        session_private_key: String,
        server_public_key: String,
        spend_private_key: String,
    ) -> Self {
        Self {
            user_id,
            session_id,
            session_private_key,
            server_public_key,
            spend_private_key,
            is_spend_private_sum: false,
        }
    }

    pub fn new_from_file(path: &str) -> Result<Self, Error> {
        let file = std::fs::read(path).unwrap();
        let safe_user: SafeUser = serde_json::from_slice(&file).unwrap();
        Ok(safe_user)
    }

    pub fn new_from_env() -> Result<Self, Error> {
        Self::new_from_env_str("")
    }

    pub fn new_from_env_str(env: &str) -> Result<Self, Error> {
        let env = if env.is_empty() {
            "TEST_KEYSTORE_PATH"
        } else {
            env
        };
        let env = std::env::var(env).unwrap();
        let path = std::path::Path::new(&env);
        Self::new_from_file(path.to_str().unwrap())
    }
}

impl GhostKeys {
    pub fn keys_slice(&self) -> Vec<crypto::Key> {
        self.keys
            .iter()
            .map(|k| crypto::Key::from_string(k).expect("Invalid key format"))
            .collect()
    }
}

pub async fn request_safe_ghost_keys(
    gkr: &[GhostKeyRequest],
    user: &SafeUser,
) -> Result<Vec<GhostKeys>, Error> {
    let data = serde_json::to_vec(gkr)?;
    let path = "/safe/keys";

    let token = auth::sign_authentication_token(
        reqwest::Method::POST.as_str(),
        path,
        &String::from_utf8_lossy(&data),
        user,
    )?;

    let response = HTTP_CLIENT
        .post(DEFAULT_API_HOST.to_string() + path)
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .header(CONTENT_TYPE, "application/json")
        .header(USER_AGENT, DEFAULT_USER_AGENT)
        .json(&gkr)
        .send()
        .await?;

    let body = response.bytes().await?;

    let parsed: ApiResponse<Vec<GhostKeys>> = serde_json::from_slice(&body)?;

    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }

    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain user data".to_string()))
}

fn spend_signing_key(user: &SafeUser) -> Result<ed25519_dalek::SigningKey, Error> {
    let key_bytes = hex::decode(&user.spend_private_key)?;
    let seed = match key_bytes.len() {
        32 => key_bytes,
        64 => key_bytes[..32].to_vec(),
        _ => return Err(Error::Input("invalid spend private key length".to_string())),
    };
    let seed_bytes: [u8; 32] = seed
        .try_into()
        .map_err(|_| Error::Input("invalid spend private key bytes".to_string()))?;
    Ok(ed25519_dalek::SigningKey::from_bytes(&seed_bytes))
}

fn spend_public_key_hex(user: &SafeUser) -> Result<String, Error> {
    let signing_key = spend_signing_key(user)?;
    Ok(hex::encode(signing_key.verifying_key().to_bytes()))
}

fn sign_user_id_base64(user: &SafeUser) -> Result<String, Error> {
    let signing_key = spend_signing_key(user)?;
    let mut hasher = Sha256::new();
    hasher.update(user.user_id.as_bytes());
    let digest = hasher.finalize();
    let signature = signing_key.sign(&digest);
    Ok(URL_SAFE_NO_PAD.encode(signature.to_bytes()))
}

fn unix_nanos() -> Result<u64, Error> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Error::Server(e.to_string()))?
        .as_nanos() as u64)
}

pub async fn register_safe_user(safe_user: &SafeUser) -> Result<User, Error> {
    let public_key = spend_public_key_hex(safe_user)?;
    let signature = sign_user_id_base64(safe_user)?;
    let timestamp = unix_nanos()?;

    let tip_body = tip_body_for_sequencer_register(&safe_user.user_id, &public_key);
    let tip_signature = sign_tip_body(
        &tip_body,
        &safe_user.spend_private_key,
        safe_user.is_spend_private_sum,
    )?;
    let pin_base64 = encrypt_ed25519_pin(&tip_signature, timestamp, safe_user)?;

    let data = serde_json::json!({
        "public_key": public_key,
        "signature": signature,
        "pin_base64": pin_base64,
    });
    let data_str = data.to_string();

    let path = "/safe/users";
    let token = auth::sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<User> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain user data".to_string()))
}

pub async fn verify_tip(safe_user: &SafeUser) -> Result<User, Error> {
    let timestamp = unix_nanos()? as i64;
    let tip_body = tip_body_for_verify(timestamp);
    let tip_signature = sign_tip_body(
        &tip_body,
        &safe_user.spend_private_key,
        safe_user.is_spend_private_sum,
    )?;
    let pin_base64 = encrypt_ed25519_pin(&tip_signature, timestamp as u64, safe_user)?;

    let data = serde_json::json!({
        "pin_base64": pin_base64,
        "timestamp": timestamp,
    });
    let data_str = data.to_string();
    let path = "/pin/verify";
    let token = auth::sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<User> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain user data".to_string()))
}

#[derive(Debug, Deserialize)]
pub struct OAuthError {
    pub code: i32,
    pub description: String,
}

pub mod crypto {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Key {
        pub value: String,
    }

    impl Key {
        pub fn from_string(s: &str) -> Result<Self, Box<dyn std::error::Error>> {
            Ok(Self {
                value: s.to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;
    use ed25519_dalek::SigningKey;

    #[tokio::test]
    async fn test_new_from_env() {
        if env::var("TEST_KEYSTORE_PATH").is_err() {
            println!("TEST_KEYSTORE_PATH is not set");
            return;
        }
        let safe_user = SafeUser::new_from_env().expect("Failed to init user from env");
        println!("{:?}", safe_user);
    }

    #[test]
    fn test_spend_public_key_hex() {
        let seed = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
        let safe_user = SafeUser::new(
            "user-id".to_string(),
            "session-id".to_string(),
            "session-private".to_string(),
            "server-public".to_string(),
            seed.to_string(),
        );
        let public_key = spend_public_key_hex(&safe_user).expect("pub");
        let signing_key = SigningKey::from_bytes(&hex::decode(seed).unwrap().try_into().unwrap());
        assert_eq!(
            public_key,
            hex::encode(signing_key.verifying_key().to_bytes())
        );
    }

    #[test]
    fn test_sign_user_id_base64() {
        let seed = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
        let safe_user = SafeUser::new(
            "user-id".to_string(),
            "session-id".to_string(),
            "session-private".to_string(),
            "server-public".to_string(),
            seed.to_string(),
        );
        let signature = sign_user_id_base64(&safe_user).expect("sig");
        let decoded = URL_SAFE_NO_PAD.decode(signature).expect("decode");
        assert_eq!(decoded.len(), 64);
    }
}
