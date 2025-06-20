use crate::{
    auth::sign_authentication_token,
    error::Error,
    request::{ApiResponse, DEFAULT_API_HOST, DEFAULT_USER_AGENT, HTTP_CLIENT, request_with_id},
    safe::SafeUser,
};
use reqwest::{
    // Client,
    header::{AUTHORIZATION, CONTENT_TYPE, USER_AGENT},
};
use serde::Deserialize;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::safe::SafeUser;

    #[tokio::test]
    async fn test_get_user() {
        let safe_user = SafeUser::new_from_env().unwrap();
        // match get_user(&safe_user, "").await {
        //     Ok(user) => {
        //         println!("Successfully retrieved user: {:#?}", user);
        //         assert_eq!(user.user_id, "fcb87491-4fa0-4c2f-b387-262b63cbc112");
        //     }
        //     Err(e) => {
        //         panic!("Failed to get user: {}", e);
        //     }
        // }

        match request_user_me(&safe_user).await {
            Ok(user) => {
                println!("Successfully retrieved user: {:#?}", user);
            }
            Err(e) => {
                panic!("Failed to get user: {}", e);
            }
        }
    }
}
