use serde::{Deserialize, Serialize};
use url::form_urlencoded;

use crate::{
    app::App,
    auth::sign_authentication_token,
    error::Error,
    request::{ApiResponse, request},
    safe::SafeUser,
};

pub const SCOPE_PROFILE_READ: &str = "PROFILE:READ";
pub const SCOPE_ASSETS_READ: &str = "ASSETS:READ";
pub const SCOPE_PHONE_READ: &str = "PHONE:READ";
pub const SCOPE_CONTACTS_READ: &str = "CONTACTS:READ";
pub const SCOPE_MESSAGES_REPRESENT: &str = "MESSAGES:REPRESENT";
pub const SCOPE_SNAPSHOTS_READ: &str = "SNAPSHOTS:READ";
pub const SCOPE_CIRCLES_READ: &str = "CIRCLES:READ";
pub const SCOPE_CIRCLES_WRITE: &str = "CIRCLES:WRITE";

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccessTokenRequest {
    pub client_id: String,
    pub code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ed25519: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_verifier: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccessTokenResponse {
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub access_token: Option<String>,
    #[serde(default)]
    pub authorization_id: Option<String>,
    #[serde(default)]
    pub ed25519: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizeRequest {
    pub authorization_id: String,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pin_base64: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Authorization {
    #[serde(default, rename = "type")]
    pub type_name: Option<String>,
    pub authorization_id: String,
    #[serde(default)]
    pub authorization_code: Option<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub code_id: Option<String>,
    #[serde(default)]
    pub app: Option<App>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub accessed_at: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RevokeAuthorizeRequest {
    pub client_id: String,
}

pub async fn get_token(request_body: &AccessTokenRequest) -> Result<AccessTokenResponse, Error> {
    let path = "/oauth/token";
    let data_str = serde_json::to_string(request_body)?;
    let body = request("POST", path, data_str.as_bytes(), "").await?;

    let parsed: ApiResponse<AccessTokenResponse> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain OAuth token data".to_string())
    })
}

pub async fn authorize(
    request_body: &AuthorizeRequest,
    safe_user: &SafeUser,
) -> Result<Authorization, Error> {
    let path = "/oauth/authorize";
    let data_str = serde_json::to_string(request_body)?;
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<Authorization> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain authorization data".to_string())
    })
}

pub async fn list_authorizations(
    app_id: Option<&str>,
    safe_user: &SafeUser,
) -> Result<Vec<Authorization>, Error> {
    let path = authorizations_path(app_id);
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    let parsed: ApiResponse<Vec<Authorization>> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain authorization data".to_string())
    })
}

pub async fn revoke_authorize(client_id: &str, safe_user: &SafeUser) -> Result<(), Error> {
    let path = "/oauth/cancel";
    let data_str = serde_json::to_string(&RevokeAuthorizeRequest {
        client_id: client_id.to_string(),
    })?;
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<serde_json::Value> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    Ok(())
}

fn authorizations_path(app_id: Option<&str>) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    if let Some(app_id) = app_id
        && !app_id.is_empty()
    {
        serializer.append_pair("app", app_id);
    }
    let query = serializer.finish();
    if query.is_empty() {
        "/authorizations".to_string()
    } else {
        format!("/authorizations?{query}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_token_request_omits_optional_fields() {
        let request = AccessTokenRequest {
            client_id: "client-id".to_string(),
            code: "code".to_string(),
            ..Default::default()
        };
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["client_id"], "client-id");
        assert!(value.get("client_secret").is_none());
        assert!(value.get("code_verifier").is_none());
        assert!(value.get("ed25519").is_none());
    }

    #[test]
    fn test_authorize_request_serialization() {
        let request = AuthorizeRequest {
            authorization_id: "authorization-id".to_string(),
            scopes: vec![
                SCOPE_PROFILE_READ.to_string(),
                SCOPE_ASSETS_READ.to_string(),
            ],
            pin_base64: Some("pin".to_string()),
        };
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["authorization_id"], "authorization-id");
        assert_eq!(value["scopes"][0], SCOPE_PROFILE_READ);
        assert_eq!(value["pin_base64"], "pin");
    }

    #[test]
    fn test_authorizations_path() {
        assert_eq!(authorizations_path(None), "/authorizations");
        assert_eq!(
            authorizations_path(Some("app id")),
            "/authorizations?app=app+id"
        );
    }
}
