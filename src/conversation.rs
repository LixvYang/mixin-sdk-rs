use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::sign_authentication_token,
    error::Error,
    request::{ApiResponse, request},
    safe::SafeUser,
    utils::{group_conversation_id, unique_conversation_id},
};

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct ParticipantSession {
    #[serde(rename = "type")]
    pub session_type: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub public_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Participant {
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Conversation {
    pub conversation_id: String,
    #[serde(default)]
    pub creator_id: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub announcement: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub code_url: Option<String>,
    #[serde(default)]
    pub participants: Option<Vec<Participant>>,
    #[serde(default)]
    pub participant_sessions: Option<Vec<ParticipantSession>>,
}

#[derive(Debug, Serialize)]
struct ConversationCreateRequest {
    category: String,
    conversation_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    announcement: Option<String>,
    participants: Vec<Participant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    random_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct MuteRequest {
    duration: i64,
}

pub async fn create_contact_conversation(
    participant_id: &str,
    safe_user: &SafeUser,
) -> Result<Conversation, Error> {
    let conversation_id = unique_conversation_id(&safe_user.user_id, participant_id);
    let participants = vec![Participant {
        user_id: participant_id.to_string(),
        role: None,
        created_at: None,
    }];
    create_conversation(
        "CONTACT",
        &conversation_id,
        None,
        None,
        participants,
        None,
        safe_user,
    )
    .await
}

pub async fn create_group_conversation(
    name: &str,
    announcement: &str,
    participants: Vec<Participant>,
    safe_user: &SafeUser,
) -> Result<Conversation, Error> {
    let random_id = Uuid::new_v4().to_string();
    let participant_ids: Vec<String> = participants.iter().map(|p| p.user_id.clone()).collect();
    let conversation_id = group_conversation_id(&safe_user.user_id, name, &participant_ids, &random_id);

    create_conversation(
        "GROUP",
        &conversation_id,
        Some(name.to_string()),
        Some(announcement.to_string()),
        participants,
        Some(random_id),
        safe_user,
    )
    .await
}

pub async fn create_conversation(
    category: &str,
    conversation_id: &str,
    name: Option<String>,
    announcement: Option<String>,
    participants: Vec<Participant>,
    random_id: Option<String>,
    safe_user: &SafeUser,
) -> Result<Conversation, Error> {
    if category == "CONTACT" && participants.len() != 1 {
        return Err(Error::Input(format!(
            "CONTACT conversation requires 1 participant, got {}",
            participants.len()
        )));
    }
    let data = ConversationCreateRequest {
        category: category.to_string(),
        conversation_id: conversation_id.to_string(),
        name,
        announcement,
        participants,
        random_id,
    };
    let data_str = serde_json::to_string(&data)?;
    let path = "/conversations";
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<Conversation> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain conversation data".to_string()))
}

pub async fn get_conversation(conversation_id: &str, safe_user: &SafeUser) -> Result<Conversation, Error> {
    let path = format!("/conversations/{conversation_id}");
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    let parsed: ApiResponse<Conversation> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain conversation data".to_string()))
}

pub async fn join_conversation(conversation_id: &str, safe_user: &SafeUser) -> Result<Conversation, Error> {
    let path = format!("/conversations/{conversation_id}/join");
    let token = sign_authentication_token("POST", &path, "", safe_user)?;
    let body = request("POST", &path, &[], &token).await?;
    let parsed: ApiResponse<Conversation> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain conversation data".to_string()))
}

pub async fn rotate_conversation(conversation_id: &str, safe_user: &SafeUser) -> Result<Conversation, Error> {
    let path = format!("/conversations/{conversation_id}/rotate");
    let token = sign_authentication_token("POST", &path, "", safe_user)?;
    let body = request("POST", &path, &[], &token).await?;
    let parsed: ApiResponse<Conversation> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain conversation data".to_string()))
}

pub async fn update_participants(
    conversation_id: &str,
    action: &str,
    participants: Vec<Participant>,
    safe_user: &SafeUser,
) -> Result<Conversation, Error> {
    let path = format!("/conversations/{conversation_id}/participants/{action}");
    let data_str = serde_json::to_string(&participants)?;
    let token = sign_authentication_token("POST", &path, &data_str, safe_user)?;
    let body = request("POST", &path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<Conversation> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain conversation data".to_string()))
}

pub async fn mute_conversation(
    conversation_id: &str,
    duration: i64,
    safe_user: &SafeUser,
) -> Result<Conversation, Error> {
    let path = format!("/conversations/{conversation_id}/mute");
    let data = MuteRequest { duration };
    let data_str = serde_json::to_string(&data)?;
    let token = sign_authentication_token("POST", &path, &data_str, safe_user)?;
    let body = request("POST", &path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<Conversation> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain conversation data".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_conversation_payload() {
        let participants = vec![Participant {
            user_id: "user-id".to_string(),
            role: Some("MEMBER".to_string()),
            created_at: None,
        }];
        let request = ConversationCreateRequest {
            category: "GROUP".to_string(),
            conversation_id: "conversation-id".to_string(),
            name: Some("Group".to_string()),
            announcement: Some("Hello".to_string()),
            participants,
            random_id: Some("random-id".to_string()),
        };
        let value: serde_json::Value = serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["category"], "GROUP");
        assert_eq!(value["conversation_id"], "conversation-id");
        assert_eq!(value["name"], "Group");
        assert_eq!(value["announcement"], "Hello");
        assert_eq!(value["random_id"], "random-id");
        assert_eq!(value["participants"][0]["user_id"], "user-id");
    }
}
