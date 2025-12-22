use crate::{
    auth::sign_authentication_token,
    error::Error,
    request::{
        ApiResponse, DEFAULT_API_HOST, DEFAULT_USER_AGENT, HTTP_CLIENT, request, request_with_id,
        simple_request,
    },
    safe::SafeUser,
};
use reqwest::{
    // Client,
    header::{AUTHORIZATION, CONTENT_TYPE, USER_AGENT},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Default)]
pub struct User {
    pub user_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub pin_token: Option<String>,
    #[serde(default)]
    pub pin_token_base64: Option<String>,
    #[serde(default)]
    pub identity_number: Option<String>,
    #[serde(default)]
    pub has_safe: Option<bool>,
    #[serde(default)]
    pub tip_key_base64: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub device_status: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub membership: Option<Membership>,
    #[serde(default)]
    pub app_id: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub biography: Option<String>,
    #[serde(default)]
    pub relationship: Option<String>,
    #[serde(default)]
    pub mute_until: Option<String>,
    #[serde(default)]
    pub is_verified: Option<bool>,
    #[serde(default)]
    pub is_scam: Option<bool>,
    #[serde(default)]
    pub is_deactivated: Option<bool>,
    #[serde(default)]
    pub code_id: Option<String>,
    #[serde(default)]
    pub code_url: Option<String>,
    #[serde(default)]
    pub features: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct Membership {
    #[serde(default)]
    pub plan: Option<String>,
    #[serde(default)]
    pub expired_at: Option<String>,
}

pub static RELATIONSHIP_ACTION_ADD: &str = "ADD";
pub static RELATIONSHIP_ACTION_UPDATE: &str = "UPDATE";
pub static RELATIONSHIP_ACTION_REMOVE: &str = "REMOVE";
pub static RELATIONSHIP_ACTION_BLOCK: &str = "BLOCK";
pub static RELATIONSHIP_ACTION_UNBLOCK: &str = "UNBLOCK";

pub static PREFERENCE_SOURCE_ALL: &str = "EVERYBODY";
pub static PREFERENCE_SOURCE_CONTACTS: &str = "CONTACTS";
pub static PREFERENCE_SOURCE_NO_BODY: &str = "NOBODY";

#[derive(Debug, Serialize)]
struct PreferenceUpdate<'a> {
    receive_message_source: &'a str,
    accept_conversation_source: &'a str,
    fiat_currency: &'a str,
    transfer_notification_threshold: &'a f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    transfer_confirmation_threshold: Option<&'a f64>,
}

#[derive(Debug, Serialize)]
struct RelationshipRequest<'a> {
    user_id: &'a str,
    action: &'a str,
}

pub async fn create_user_simple(session_secret: &str, full_name: &str) -> Result<User, Error> {
    let data = serde_json::json!({
        "session_secret": session_secret,
        "full_name": full_name,
    });

    let body = simple_request("POST", "/users", data.to_string().as_bytes()).await?;
    let parsed: ApiResponse<User> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain user data".to_string()))
        .map_err(|e| Error::DataNotFound(e.to_string()))
}

pub async fn create_user_with_phone(
    session_secret: &str,
    full_name: &str,
    safe_user: &SafeUser,
) -> Result<User, Error> {
    let data = serde_json::json!({
        "session_secret": session_secret,
        "full_name": full_name,
    });

    let data_str = data.to_string();
    let token = sign_authentication_token("POST", "/users", &data_str, safe_user)?;
    let body = request("POST", "/users", data_str.as_bytes(), &token).await?;
    let parsed: ApiResponse<User> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain user data".to_string()))
        .map_err(|e| Error::DataNotFound(e.to_string()))
}

pub async fn get_user(safe_user: &SafeUser, user_id: &str) -> Result<User, Error> {
    let path = "/users/".to_string() + user_id;

    // 1. Sign authentication token
    let token = sign_authentication_token("GET", &path, "", safe_user)?;

    // 2. Create a stateless HTTP client and send the request
    //  let client = Client::new();
    let uri = DEFAULT_API_HOST.to_string() + &path;

    let response = HTTP_CLIENT
        .get(&uri)
        .header(AUTHORIZATION, "Bearer ".to_string() + &token)
        .header(CONTENT_TYPE, "application/json")
        .header(USER_AGENT, DEFAULT_USER_AGENT)
        .send()
        .await?;

    // 3. Read the response body
    let body = response.bytes().await?;

    // 4. Deserialize into a generic response wrapper
    let parsed: ApiResponse<User> = serde_json::from_slice(&body)?;

    // 5. Check for an API-level error in the response
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }

    // 6. Extract and return the user data
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain user data".to_string()))
        .map_err(|e| Error::DataNotFound(e.to_string()))
}

pub async fn search_user(query: &str, safe_user: &SafeUser) -> Result<Vec<User>, Error> {
    let path = "/search/".to_string() + query;
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;
    let parsed: ApiResponse<Vec<User>> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain user data".to_string()))
        .map_err(|e| Error::DataNotFound(e.to_string()))
}

pub async fn user_me_with_request_id(access_token: &str, request_id: &str) -> Result<User, Error> {
    let method = "GET";
    let path = "/safe/me";
    let response = request_with_id(method, path, &[], access_token, request_id.to_string()).await?;
    let parsed: ApiResponse<User> = serde_json::from_slice(&response)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain user data".to_string()))
        .map_err(|e| Error::DataNotFound(e.to_string()))
}

pub async fn user_me(access_token: &str) -> Result<User, Error> {
    return user_me_with_request_id(access_token, &Uuid::new_v4().to_string()).await;
}

pub async fn request_user_me(safe_user: &SafeUser) -> Result<User, Error> {
    let path = "/safe/me";
    let token = sign_authentication_token("GET", path, "", safe_user)?;
    return user_me(&token).await;
}

pub async fn update_user_me(
    full_name: &str,
    avatar_base64: &str,
    safe_user: &SafeUser,
) -> Result<User, Error> {
    let data = serde_json::json!({
        "full_name": full_name,
        "avatar_base64": avatar_base64,
    });

    let path = "/me";
    let data_str = data.to_string();
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;
    let parsed: ApiResponse<User> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain user data".to_string()))
        .map_err(|e| Error::DataNotFound(e.to_string()))
}

pub async fn update_preference(
    message_source: &str,
    conversation_source: &str,
    currency: &str,
    threshold: &f64,
    confirmation_threshold: Option<&f64>,
    safe_user: &SafeUser,
) -> Result<User, Error> {
    let data = PreferenceUpdate {
        receive_message_source: message_source,
        accept_conversation_source: conversation_source,
        fiat_currency: currency,
        transfer_notification_threshold: threshold,
        transfer_confirmation_threshold: confirmation_threshold,
    };

    let path = "/me/preferences";
    let data_str = serde_json::to_string(&data)?;
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;
    let parsed: ApiResponse<User> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain user data".to_string()))
        .map_err(|e| Error::DataNotFound(e.to_string()))
}

pub async fn relationship(
    user_id: &str,
    action: &str,
    safe_user: &SafeUser,
) -> Result<User, Error> {
    let data = RelationshipRequest { user_id, action };
    let path = "/relationships";
    let data_str = serde_json::to_string(&data)?;
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;
    let parsed: ApiResponse<User> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain user data".to_string()))
        .map_err(|e| Error::DataNotFound(e.to_string()))
}

pub async fn get_friends(safe_user: &SafeUser) -> Result<Vec<User>, Error> {
    let path = "/friends";
    let token = sign_authentication_token("GET", path, "", safe_user)?;
    let body = request("GET", path, &[], &token).await?;
    let parsed: ApiResponse<Vec<User>> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain user data".to_string()))
        .map_err(|e| Error::DataNotFound(e.to_string()))
}

pub async fn get_blocking_users(safe_user: &SafeUser) -> Result<Vec<User>, Error> {
    let path = "/blocking_users";
    let token = sign_authentication_token("GET", path, "", safe_user)?;
    let body = request("GET", path, &[], &token).await?;
    let parsed: ApiResponse<Vec<User>> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain user data".to_string()))
        .map_err(|e| Error::DataNotFound(e.to_string()))
}

pub async fn get_users(safe_user: &SafeUser, user_ids: &[String]) -> Result<Vec<User>, Error> {
    let data = serde_json::json!({
        "user_ids": user_ids,
    });
    let path = "/users/fetch";
    let data_str = data.to_string();
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;
    let parsed: ApiResponse<Vec<User>> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain user data".to_string()))
        .map_err(|e| Error::DataNotFound(e.to_string()))
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;
    use crate::safe::SafeUser;

    #[tokio::test]
    async fn test_get_user() {
        if env::var("TEST_KEYSTORE_PATH").is_err() {
            println!("TEST_KEYSTORE_PATH is not set");
            return;
        }
        let safe_user = SafeUser::new_from_env().unwrap();
        match request_user_me(&safe_user).await {
            Ok(user) => {
                println!("Successfully retrieved user: {:#?}", user);
            }
            Err(e) => {
                panic!("Failed to get user: {}", e);
            }
        }
    }

    #[test]
    fn test_preference_update_serialization() {
        let data = PreferenceUpdate {
            receive_message_source: PREFERENCE_SOURCE_ALL,
            accept_conversation_source: PREFERENCE_SOURCE_CONTACTS,
            fiat_currency: "USD",
            transfer_notification_threshold: &10.0,
            transfer_confirmation_threshold: None,
        };
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&data).unwrap()).unwrap();
        assert_eq!(value["receive_message_source"], "EVERYBODY");
        assert_eq!(value["accept_conversation_source"], "CONTACTS");
        assert_eq!(value["fiat_currency"], "USD");
        assert_eq!(value["transfer_notification_threshold"], 10.0);
        assert!(value.get("transfer_confirmation_threshold").is_none());
    }

    #[test]
    fn test_relationship_request_serialization() {
        let data = RelationshipRequest {
            user_id: "user-id",
            action: RELATIONSHIP_ACTION_ADD,
        };
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&data).unwrap()).unwrap();
        assert_eq!(value["user_id"], "user-id");
        assert_eq!(value["action"], "ADD");
    }
}
