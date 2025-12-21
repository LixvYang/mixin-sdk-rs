use serde::{Deserialize, Serialize};

use crate::{
    auth::sign_authentication_token,
    error::Error,
    request::{ApiResponse, request},
    safe::SafeUser,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageRequest {
    pub conversation_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient_id: Option<String>,
    pub message_id: String,
    pub category: String,
    pub data_base64: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub representative_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_message_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReceiptAcknowledgementRequest {
    pub message_id: String,
    pub status: String,
}

pub async fn post_messages(messages: &[MessageRequest], safe_user: &SafeUser) -> Result<(), Error> {
    let data_str = serde_json::to_string(messages)?;
    let path = "/messages";
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<serde_json::Value> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    Ok(())
}

pub async fn post_message(message: MessageRequest, safe_user: &SafeUser) -> Result<(), Error> {
    post_messages(&[message], safe_user).await
}

pub async fn post_acknowledgements(
    requests: &[ReceiptAcknowledgementRequest],
    safe_user: &SafeUser,
) -> Result<(), Error> {
    let data_str = serde_json::to_string(requests)?;
    let path = "/acknowledgements";
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<serde_json::Value> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_request_serialization() {
        let request = MessageRequest {
            conversation_id: "conversation-id".to_string(),
            recipient_id: None,
            message_id: "message-id".to_string(),
            category: "PLAIN_TEXT".to_string(),
            data_base64: "SGVsbG8=".to_string(),
            representative_id: None,
            quote_message_id: Some("quote-id".to_string()),
        };
        let value: serde_json::Value = serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["conversation_id"], "conversation-id");
        assert_eq!(value["message_id"], "message-id");
        assert_eq!(value["category"], "PLAIN_TEXT");
        assert_eq!(value["data_base64"], "SGVsbG8=");
        assert_eq!(value["quote_message_id"], "quote-id");
        assert!(value.get("recipient_id").is_none());
    }

    #[test]
    fn test_acknowledgement_serialization() {
        let ack = ReceiptAcknowledgementRequest {
            message_id: "message-id".to_string(),
            status: "READ".to_string(),
        };
        let value: serde_json::Value = serde_json::from_str(&serde_json::to_string(&ack).unwrap()).unwrap();
        assert_eq!(value["message_id"], "message-id");
        assert_eq!(value["status"], "READ");
    }
}
