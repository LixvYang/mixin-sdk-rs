use serde::{Deserialize, Serialize};
use url::form_urlencoded;

use crate::{
    auth::sign_authentication_token,
    error::Error,
    request::{ApiResponse, request},
    safe::SafeUser,
};

pub const CIRCLE_ACTION_ADD: &str = "ADD";
pub const CIRCLE_ACTION_REMOVE: &str = "REMOVE";

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Circle {
    #[serde(default, rename = "type")]
    pub type_name: Option<String>,
    pub circle_id: String,
    pub user_id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CircleConversation {
    #[serde(default, rename = "type")]
    pub type_name: Option<String>,
    pub circle_id: String,
    pub conversation_id: String,
    pub user_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CircleConversationQuery {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
struct CircleNameRequest<'a> {
    name: &'a str,
}

#[derive(Debug, Serialize)]
struct CircleActionRequest<'a> {
    #[serde(rename = "circleID")]
    circle_id: &'a str,
    action: &'a str,
}

pub async fn get_circle(circle_id: &str, safe_user: &SafeUser) -> Result<Circle, Error> {
    let path = format!("/circles/{circle_id}");
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;
    parse_one(body, "circle").await
}

pub async fn list_circles(safe_user: &SafeUser) -> Result<Vec<Circle>, Error> {
    let path = "/circles";
    let token = sign_authentication_token("GET", path, "", safe_user)?;
    let body = request("GET", path, &[], &token).await?;

    let parsed: ApiResponse<Vec<Circle>> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain circle data".to_string()))
}

pub async fn list_circle_conversations(
    circle_id: &str,
    query: &CircleConversationQuery,
    safe_user: &SafeUser,
) -> Result<Vec<CircleConversation>, Error> {
    let path = circle_conversations_path(circle_id, query);
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    let parsed: ApiResponse<Vec<CircleConversation>> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain circle conversation data".to_string())
    })
}

pub async fn create_circle(name: &str, safe_user: &SafeUser) -> Result<Circle, Error> {
    let path = "/circles";
    let data_str = serde_json::to_string(&CircleNameRequest { name })?;
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;
    parse_one(body, "circle").await
}

pub async fn update_circle(
    circle_id: &str,
    name: &str,
    safe_user: &SafeUser,
) -> Result<Circle, Error> {
    let path = format!("/circles/{circle_id}");
    let data_str = serde_json::to_string(&CircleNameRequest { name })?;
    let token = sign_authentication_token("POST", &path, &data_str, safe_user)?;
    let body = request("POST", &path, data_str.as_bytes(), &token).await?;
    parse_one(body, "circle").await
}

pub async fn delete_circle(circle_id: &str, safe_user: &SafeUser) -> Result<(), Error> {
    mutate_empty(&format!("/circles/{circle_id}/delete"), safe_user).await
}

pub async fn add_user_to_circle(
    user_id: &str,
    circle_id: &str,
    safe_user: &SafeUser,
) -> Result<Vec<Circle>, Error> {
    update_user_circle(user_id, circle_id, CIRCLE_ACTION_ADD, safe_user).await
}

pub async fn remove_user_from_circle(
    user_id: &str,
    circle_id: &str,
    safe_user: &SafeUser,
) -> Result<Vec<Circle>, Error> {
    update_user_circle(user_id, circle_id, CIRCLE_ACTION_REMOVE, safe_user).await
}

pub async fn update_user_circle(
    user_id: &str,
    circle_id: &str,
    action: &str,
    safe_user: &SafeUser,
) -> Result<Vec<Circle>, Error> {
    let path = format!("/users/{user_id}/circles");
    post_circle_action(&path, circle_id, action, safe_user).await
}

pub async fn add_conversation_to_circle(
    conversation_id: &str,
    circle_id: &str,
    safe_user: &SafeUser,
) -> Result<Vec<Circle>, Error> {
    update_conversation_circle(conversation_id, circle_id, CIRCLE_ACTION_ADD, safe_user).await
}

pub async fn remove_conversation_from_circle(
    conversation_id: &str,
    circle_id: &str,
    safe_user: &SafeUser,
) -> Result<Vec<Circle>, Error> {
    update_conversation_circle(conversation_id, circle_id, CIRCLE_ACTION_REMOVE, safe_user).await
}

pub async fn update_conversation_circle(
    conversation_id: &str,
    circle_id: &str,
    action: &str,
    safe_user: &SafeUser,
) -> Result<Vec<Circle>, Error> {
    let path = format!("/conversations/{conversation_id}/circles");
    post_circle_action(&path, circle_id, action, safe_user).await
}

async fn post_circle_action(
    path: &str,
    circle_id: &str,
    action: &str,
    safe_user: &SafeUser,
) -> Result<Vec<Circle>, Error> {
    let data_str = serde_json::to_string(&CircleActionRequest { circle_id, action })?;
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<Vec<Circle>> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain circle data".to_string()))
}

async fn mutate_empty(path: &str, safe_user: &SafeUser) -> Result<(), Error> {
    let token = sign_authentication_token("POST", path, "", safe_user)?;
    let body = request("POST", path, &[], &token).await?;
    let parsed: ApiResponse<serde_json::Value> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    Ok(())
}

async fn parse_one(body: Vec<u8>, name: &str) -> Result<Circle, Error> {
    let parsed: ApiResponse<Circle> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound(format!("API response did not contain {name} data")))
}

fn circle_conversations_path(circle_id: &str, query: &CircleConversationQuery) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    if let Some(offset) = &query.offset
        && !offset.is_empty()
    {
        serializer.append_pair("offset", offset);
    }
    if let Some(limit) = query.limit {
        serializer.append_pair("limit", &limit.to_string());
    }
    let query = serializer.finish();
    if query.is_empty() {
        format!("/circles/{circle_id}/conversations")
    } else {
        format!("/circles/{circle_id}/conversations?{query}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_action_request_uses_node_field_name() {
        let request = CircleActionRequest {
            circle_id: "circle-id",
            action: CIRCLE_ACTION_ADD,
        };
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["circleID"], "circle-id");
        assert_eq!(value["action"], CIRCLE_ACTION_ADD);
    }

    #[test]
    fn test_circle_conversations_path() {
        let query = CircleConversationQuery {
            offset: Some("2026-05-16T00:00:00Z".to_string()),
            limit: Some(50),
        };
        assert_eq!(
            circle_conversations_path("circle-id", &query),
            "/circles/circle-id/conversations?offset=2026-05-16T00%3A00%3A00Z&limit=50"
        );
    }
}
