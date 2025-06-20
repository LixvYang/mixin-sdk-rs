use once_cell::sync::Lazy;
use reqwest::{
    Client,
    header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT},
};
use serde::Deserialize;
use std::sync::Mutex;
use std::time::Duration;
use uuid::Uuid;

use crate::error::Error;

pub const DEFAULT_API_HOST: &str = "https://api.mixin.one";
pub const DEFAULT_BLAZE_HOST: &str = "blaze.mixin.one";
pub const DEFAULT_USER_AGENT: &str = "Bot-API-Rust-Client";

pub static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to create HTTP client")
});

static HTTP_URI: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(DEFAULT_API_HOST.to_string()));
static BLAZE_URI: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(DEFAULT_BLAZE_HOST.to_string()));
static USER_AGENT_STR: Lazy<Mutex<String>> =
    Lazy::new(|| Mutex::new(DEFAULT_USER_AGENT.to_string()));
static UID: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));
static SID: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));
static PRIVATE_KEY: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));

#[derive(Debug, Deserialize, Default)]
pub struct ApiResponse<T> {
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ApiError {
    pub status: i32,
    pub code: i32,
    pub description: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "API Error (Status: {}, Code: {}): {}",
            self.status, self.code, self.description
        )
    }
}

impl std::error::Error for ApiError {}

pub fn with_api_key(user_id: String, session_id: String, private_key: String) {
    *UID.lock().unwrap() = user_id;
    *SID.lock().unwrap() = session_id;
    *PRIVATE_KEY.lock().unwrap() = private_key;
}

pub fn set_base_uri(base: String) {
    *HTTP_URI.lock().unwrap() = base;
}

pub fn set_blaze_uri(blaze: String) {
    *BLAZE_URI.lock().unwrap() = blaze;
}

// 设置 User Agent
pub fn set_user_agent(ua: String) {
    *USER_AGENT_STR.lock().unwrap() = ua;
}

// 发送请求
pub async fn request(
    method: &str,
    path: &str,
    body: &[u8],
    access_token: &str,
) -> Result<Vec<u8>, Error> {
    request_with_id(method, path, body, access_token, Uuid::new_v4().to_string()).await
}

pub async fn request_with_id(
    method: &str,
    path: &str,
    body: &[u8],
    access_token: &str,
    request_id: String,
) -> Result<Vec<u8>, Error> {
    let uri = format!("{}{}", *HTTP_URI.lock().unwrap(), path);

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", access_token))?,
    );
    headers.insert("X-Request-Id", HeaderValue::from_str(&request_id)?);
    headers.insert(
        USER_AGENT,
        HeaderValue::from_str(&*USER_AGENT_STR.lock().unwrap())?,
    );

    let response = HTTP_CLIENT
        .request(reqwest::Method::from_bytes(method.as_bytes())?, &uri)
        .headers(headers)
        .body(body.to_vec())
        .send()
        .await?;

    if response.status().is_server_error() {
        let error = ApiError {
            status: response.status().as_u16() as i32,
            code: 0,
            description: "Server error".to_string(),
        };
        return Err(error.into());
    }

    let body = response.bytes().await?;
    Ok(body.to_vec())
}

pub async fn simple_request(method: &str, path: &str, body: &[u8]) -> Result<Vec<u8>, Error> {
    let uri = format!("{}{}", *HTTP_URI.lock().unwrap(), path);

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    let response = HTTP_CLIENT
        .request(reqwest::Method::from_bytes(method.as_bytes())?, &uri)
        .headers(headers)
        .body(body.to_vec())
        .send()
        .await?;

    if response.status().is_server_error() {
        let error = ApiError {
            status: response.status().as_u16() as i32,
            code: 0,
            description: "Server error".to_string(),
        };
        return Err(error.into());
    }

    let body = response.bytes().await?;
    Ok(body.to_vec())
}
