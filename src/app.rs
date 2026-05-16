use serde::{Deserialize, Serialize};

use crate::{
    auth::sign_authentication_token,
    error::Error,
    pin::encrypt_ed25519_pin,
    request::{ApiResponse, request},
    safe::SafeUser,
    tip::{TIP_OWNERSHIP_TRANSFER, sign_tip_body, tip_body},
};

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct App {
    #[serde(rename = "type")]
    pub type_name: Option<String>,
    pub app_id: String,
    #[serde(default)]
    pub app_number: Option<String>,
    #[serde(default)]
    pub redirect_url: Option<String>,
    #[serde(default)]
    pub home_url: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub capabilities: Option<Vec<String>>,
    #[serde(default)]
    pub resource_patterns: Option<Vec<String>>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub creator_id: Option<String>,
    #[serde(default)]
    pub has_safe: Option<bool>,
    #[serde(default)]
    pub spend_public_key: Option<String>,
    #[serde(default)]
    pub safe_created_at: Option<String>,
    #[serde(default)]
    pub app_secret: Option<String>,
    #[serde(default)]
    pub session_secret: Option<String>,
    #[serde(default)]
    pub session_public_key: Option<String>,
    #[serde(default)]
    pub is_verified: Option<bool>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct AppRequest {
    pub redirect_uri: String,
    pub home_uri: String,
    pub name: String,
    pub description: String,
    pub icon_base64: String,
    pub category: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub resource_patterns: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct AppProperty {
    pub count: u32,
    pub price: String,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct AppBillingCost {
    pub users: String,
    pub resources: String,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct AppBilling {
    pub app_id: String,
    pub cost: AppBillingCost,
    pub credit: String,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct AppSecret {
    pub app_secret: String,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct AppSession {
    pub session_id: String,
    pub server_public_key: String,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct AppSafeSessionRequest {
    pub session_public_key: String,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct AppSafeRegistrationRequest {
    pub spend_public_key: String,
    pub signature_base64: String,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct AppRegistration {
    pub spend_public_key: String,
}

#[derive(Debug, Serialize)]
struct AppTransferRequest<'a> {
    user_id: &'a str,
    pin_base64: &'a str,
}

pub async fn get_app(app_id: &str, safe_user: &SafeUser) -> Result<App, Error> {
    let path = format!("/apps/{app_id}");
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    let parsed: ApiResponse<App> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain app data".to_string()))
}

pub async fn get_app_property(safe_user: &SafeUser) -> Result<AppProperty, Error> {
    let path = "/apps/property";
    let token = sign_authentication_token("GET", path, "", safe_user)?;
    let body = request("GET", path, &[], &token).await?;

    parse_one(body, "app property")
}

pub async fn get_app_billing(app_id: &str, safe_user: &SafeUser) -> Result<AppBilling, Error> {
    let path = format!("/safe/apps/{app_id}/billing");
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    parse_one(body, "app billing")
}

pub async fn list_favorite_apps(user_id: &str, safe_user: &SafeUser) -> Result<Vec<App>, Error> {
    let path = format!("/users/{user_id}/apps/favorite");
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    parse_many(body, "favorite app")
}

pub async fn create_app(request_body: &AppRequest, safe_user: &SafeUser) -> Result<App, Error> {
    let path = "/apps";
    let data_str = serde_json::to_string(request_body)?;
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    parse_one(body, "app")
}

pub async fn update_app(
    app_id: &str,
    request_body: &AppRequest,
    safe_user: &SafeUser,
) -> Result<App, Error> {
    let path = format!("/apps/{app_id}");
    let data_str = serde_json::to_string(request_body)?;
    let token = sign_authentication_token("POST", &path, &data_str, safe_user)?;
    let body = request("POST", &path, data_str.as_bytes(), &token).await?;

    parse_one(body, "app")
}

pub async fn update_app_secret(app_id: &str, safe_user: &SafeUser) -> Result<AppSecret, Error> {
    let path = format!("/apps/{app_id}/secret");
    let token = sign_authentication_token("POST", &path, "", safe_user)?;
    let body = request("POST", &path, &[], &token).await?;

    parse_one(body, "app secret")
}

pub async fn update_safe_app_session(
    app_id: &str,
    request_body: &AppSafeSessionRequest,
    safe_user: &SafeUser,
) -> Result<AppSession, Error> {
    let path = format!("/safe/apps/{app_id}/session");
    let data_str = serde_json::to_string(request_body)?;
    let token = sign_authentication_token("POST", &path, &data_str, safe_user)?;
    let body = request("POST", &path, data_str.as_bytes(), &token).await?;

    parse_one(body, "app session")
}

pub async fn register_safe_app(
    app_id: &str,
    request_body: &AppSafeRegistrationRequest,
    safe_user: &SafeUser,
) -> Result<AppRegistration, Error> {
    let path = format!("/safe/apps/{app_id}/register");
    let data_str = serde_json::to_string(request_body)?;
    let token = sign_authentication_token("POST", &path, &data_str, safe_user)?;
    let body = request("POST", &path, data_str.as_bytes(), &token).await?;

    parse_one(body, "app registration")
}

pub async fn favorite_app(app_id: &str, safe_user: &SafeUser) -> Result<Vec<App>, Error> {
    let path = format!("/apps/{app_id}/favorite");
    let token = sign_authentication_token("POST", &path, "", safe_user)?;
    let body = request("POST", &path, &[], &token).await?;

    parse_many(body, "favorite app")
}

pub async fn unfavorite_app(app_id: &str, safe_user: &SafeUser) -> Result<(), Error> {
    let path = format!("/apps/{app_id}/unfavorite");
    let token = sign_authentication_token("POST", &path, "", safe_user)?;
    let body = request("POST", &path, &[], &token).await?;

    let parsed: ApiResponse<serde_json::Value> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    Ok(())
}

pub async fn migrate_app(receiver_id: &str, safe_user: &SafeUser) -> Result<App, Error> {
    let body = tip_body(&(TIP_OWNERSHIP_TRANSFER.to_string() + receiver_id));
    let pin = sign_tip_body(
        &body,
        &safe_user.spend_private_key,
        safe_user.is_spend_private_sum,
    )?;
    let pin_base64 = encrypt_ed25519_pin(&pin, now_nanos()?, safe_user)?;
    let path = format!("/apps/{}/transfer", safe_user.user_id);
    let data_str = serde_json::to_string(&AppTransferRequest {
        user_id: receiver_id,
        pin_base64: &pin_base64,
    })?;
    let token = sign_authentication_token("POST", &path, &data_str, safe_user)?;
    let body = request("POST", &path, data_str.as_bytes(), &token).await?;

    parse_one(body, "app")
}

pub async fn list_apps(safe_user: &SafeUser) -> Result<Vec<App>, Error> {
    let path = "/apps";
    let token = sign_authentication_token("GET", path, "", safe_user)?;
    let body = request("GET", path, &[], &token).await?;

    let parsed: ApiResponse<Vec<App>> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain app data".to_string()))
}

fn parse_one<T: serde::de::DeserializeOwned>(body: Vec<u8>, name: &str) -> Result<T, Error> {
    let parsed: ApiResponse<T> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound(format!("API response did not contain {name} data")))
}

fn parse_many<T: serde::de::DeserializeOwned>(body: Vec<u8>, name: &str) -> Result<Vec<T>, Error> {
    parse_one(body, name)
}

fn now_nanos() -> Result<u64, Error> {
    use std::time::{SystemTime, UNIX_EPOCH};
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Error::Server(e.to_string()))?
        .as_nanos() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_deserialize() {
        let raw = r#"{
            "type": "app",
            "app_id": "app-id",
            "name": "Example",
            "creator_id": "creator-id",
            "has_safe": true
        }"#;
        let app: App = serde_json::from_str(raw).expect("app");
        assert_eq!(app.app_id, "app-id");
        assert_eq!(app.creator_id.as_deref(), Some("creator-id"));
        assert_eq!(app.has_safe, Some(true));
    }

    #[test]
    fn test_app_request_serialization() {
        let request = AppRequest {
            redirect_uri: "https://example.com/oauth".to_string(),
            home_uri: "https://example.com".to_string(),
            name: "Example".to_string(),
            description: "Description".to_string(),
            icon_base64: "icon".to_string(),
            category: "TOOLS".to_string(),
            capabilities: vec!["CONTACT".to_string()],
            resource_patterns: vec!["https://example.com/*".to_string()],
        };
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["redirect_uri"], "https://example.com/oauth");
        assert_eq!(value["home_uri"], "https://example.com");
        assert_eq!(value["capabilities"][0], "CONTACT");
    }

    #[test]
    fn test_app_transfer_request_serialization() {
        let request = AppTransferRequest {
            user_id: "receiver-id",
            pin_base64: "pin",
        };
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["user_id"], "receiver-id");
        assert_eq!(value["pin_base64"], "pin");
    }
}
