use once_cell::sync::Lazy;
use reqwest::{
    Client,
    header::{
        AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, USER_AGENT,
    },
};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct ApiError {
    #[serde(default)]
    pub status: i32,
    #[serde(default)]
    pub code: i32,
    #[serde(default)]
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

pub fn get_blaze_uri() -> String {
    BLAZE_URI.lock().unwrap().clone()
}

pub fn set_user_agent(ua: String) {
    *USER_AGENT_STR.lock().unwrap() = ua;
}

fn build_headers(access_token: Option<&str>, request_id: Option<&str>) -> Result<HeaderMap, Error> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        USER_AGENT,
        HeaderValue::from_str(&USER_AGENT_STR.lock().unwrap())?,
    );
    if let Some(token) = access_token {
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token))?,
        );
    }
    if let Some(request_id) = request_id {
        headers.insert(
            HeaderName::from_static("x-request-id"),
            HeaderValue::from_str(request_id)?,
        );
    }
    Ok(headers)
}

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

    let access_token = if access_token.is_empty() {
        None
    } else {
        Some(access_token)
    };
    let headers = build_headers(access_token, Some(&request_id))?;

    let method = reqwest::Method::from_bytes(method.as_bytes())?;
    let mut request_builder = HTTP_CLIENT.request(method.clone(), &uri).headers(headers);
    if method != reqwest::Method::GET {
        request_builder = request_builder.header(CONTENT_LENGTH, body.len());
    }
    let response = request_builder.body(body.to_vec()).send().await?;

    let status = response.status();
    let body = response.bytes().await?;
    if !status.is_success() {
        return Err(parse_http_error(status, &body).into());
    }
    Ok(body.to_vec())
}

pub async fn simple_request(method: &str, path: &str, body: &[u8]) -> Result<Vec<u8>, Error> {
    let uri = format!("{}{}", *HTTP_URI.lock().unwrap(), path);

    let headers = build_headers(None, None)?;
    let method = reqwest::Method::from_bytes(method.as_bytes())?;
    let mut request_builder = HTTP_CLIENT.request(method.clone(), &uri).headers(headers);
    if method != reqwest::Method::GET {
        request_builder = request_builder.header(CONTENT_LENGTH, body.len());
    }
    let response = request_builder.body(body.to_vec()).send().await?;

    let status = response.status();
    let body = response.bytes().await?;
    if !status.is_success() {
        return Err(parse_http_error(status, &body).into());
    }
    Ok(body.to_vec())
}

fn parse_http_error(status: reqwest::StatusCode, body: &[u8]) -> ApiError {
    if let Ok(parsed) = serde_json::from_slice::<ApiResponse<serde_json::Value>>(body)
        && let Some(mut api_error) = parsed.error
    {
        if api_error.status == 0 {
            api_error.status = status.as_u16() as i32;
        }
        return api_error;
    }

    let description = String::from_utf8_lossy(body).trim().to_string();
    ApiError {
        status: status.as_u16() as i32,
        code: status.as_u16() as i32,
        description: if description.is_empty() {
            status
                .canonical_reason()
                .unwrap_or("HTTP error")
                .to_string()
        } else {
            description
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_headers_with_request_id() {
        set_user_agent("test-agent".to_string());
        let headers = build_headers(Some("token-123"), Some("req-abc")).expect("headers");

        assert_eq!(
            headers.get(USER_AGENT).unwrap(),
            HeaderValue::from_static("test-agent")
        );
        assert_eq!(
            headers
                .get(HeaderName::from_static("x-request-id"))
                .unwrap(),
            HeaderValue::from_static("req-abc")
        );
        assert_eq!(
            headers.get(AUTHORIZATION).unwrap(),
            HeaderValue::from_static("Bearer token-123")
        );
    }

    #[test]
    fn test_parse_http_error_prefers_api_error_body() {
        let body = br#"{"error":{"status":403,"code":403,"description":"Forbidden"}}"#;
        let error = parse_http_error(reqwest::StatusCode::FORBIDDEN, body);
        assert_eq!(error.status, 403);
        assert_eq!(error.code, 403);
        assert_eq!(error.description, "Forbidden");
    }

    #[test]
    fn test_parse_http_error_uses_plain_body() {
        let error = parse_http_error(reqwest::StatusCode::NOT_FOUND, b"404 page not found\n");
        assert_eq!(error.status, 404);
        assert_eq!(error.code, 404);
        assert_eq!(error.description, "404 page not found");
    }

    #[test]
    fn test_api_error_deserializes_oauth_shape() {
        let body = r#"{"error":{"code":400,"description":"invalid grant"}}"#;
        let parsed: ApiResponse<serde_json::Value> = serde_json::from_str(body).expect("error");
        let error = parsed.error.expect("api error");
        assert_eq!(error.status, 0);
        assert_eq!(error.code, 400);
        assert_eq!(error.description, "invalid grant");
    }
}
