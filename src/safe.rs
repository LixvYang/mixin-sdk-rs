use crate::{
    auth,
    error::Error,
    request::{ApiResponse, DEFAULT_API_HOST, DEFAULT_USER_AGENT, HTTP_CLIENT},
};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde::{Deserialize, Serialize};

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
