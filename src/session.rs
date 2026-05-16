use serde::{Deserialize, Serialize};

use crate::{
    auth::sign_authentication_token,
    error::Error,
    request::{ApiResponse, request},
    safe::SafeUser,
};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct UserSession {
    pub user_id: String,
    pub session_id: String,
    pub public_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
}

pub async fn fetch_user_sessions(
    user_ids: &[String],
    safe_user: &SafeUser,
) -> Result<Vec<UserSession>, Error> {
    let data_str = serde_json::to_string(user_ids)?;
    let path = "/sessions/fetch";
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<Vec<UserSession>> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("user sessions".to_string()))
}

pub async fn fetch_user_session(
    user_id: &str,
    safe_user: &SafeUser,
) -> Result<Option<UserSession>, Error> {
    let sessions = fetch_user_sessions(&[user_id.to_string()], safe_user).await?;
    Ok(sessions.into_iter().next())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_session_deserialize() {
        let value = serde_json::json!({
            "user_id": "user-id",
            "session_id": "session-id",
            "public_key": "public-key",
            "platform": "iOS"
        });

        let session: UserSession = serde_json::from_value(value).unwrap();
        assert_eq!(session.user_id, "user-id");
        assert_eq!(session.session_id, "session-id");
        assert_eq!(session.public_key, "public-key");
        assert_eq!(session.platform.as_deref(), Some("iOS"));
    }
}
