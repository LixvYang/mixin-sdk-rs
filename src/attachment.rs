use reqwest::header::{CONTENT_TYPE, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::{
    auth::sign_authentication_token,
    error::Error,
    request::{ApiResponse, HTTP_CLIENT, request},
    safe::SafeUser,
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Attachment {
    #[serde(rename = "type")]
    pub attachment_type: String,
    pub attachment_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upload_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct UploadedAttachment {
    pub attachment_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view_url: Option<String>,
}

pub async fn create_attachment(safe_user: &SafeUser) -> Result<Attachment, Error> {
    let path = "/attachments";
    let token = sign_authentication_token("POST", path, "", safe_user)?;
    let body = request("POST", path, &[], &token).await?;
    parse_attachment_response(&body, "attachment")
}

pub async fn fetch_attachment(
    attachment_id: &str,
    safe_user: &SafeUser,
) -> Result<Attachment, Error> {
    let path = format!("/attachments/{attachment_id}");
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;
    parse_attachment_response(&body, attachment_id)
}

pub async fn upload_attachment_to_url(
    upload_url: &str,
    bytes: impl Into<Vec<u8>>,
) -> Result<(), Error> {
    let response = HTTP_CLIENT
        .put(upload_url)
        .header(
            HeaderName::from_static("x-amz-acl"),
            HeaderValue::from_static("public-read"),
        )
        .header(
            CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        )
        .body(bytes.into())
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(Error::Server(format!(
            "attachment upload failed: {status}: {body}"
        )));
    }

    Ok(())
}

pub async fn upload_attachment_bytes(
    bytes: impl Into<Vec<u8>>,
    safe_user: &SafeUser,
) -> Result<UploadedAttachment, Error> {
    let attachment = create_attachment(safe_user).await?;
    let upload_url = attachment
        .upload_url
        .as_deref()
        .ok_or_else(|| Error::DataNotFound("attachment.upload_url".to_string()))?;

    upload_attachment_to_url(upload_url, bytes).await?;

    Ok(UploadedAttachment {
        attachment_id: attachment.attachment_id,
        view_url: attachment.view_url,
    })
}

pub async fn upload_attachment_file<P: AsRef<Path>>(
    path: P,
    safe_user: &SafeUser,
) -> Result<UploadedAttachment, Error> {
    let bytes = tokio::fs::read(path).await?;
    upload_attachment_bytes(bytes, safe_user).await
}

fn parse_attachment_response(body: &[u8], label: &str) -> Result<Attachment, Error> {
    let parsed: ApiResponse<Attachment> = serde_json::from_slice(body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound(label.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attachment_deserialize() {
        let value = serde_json::json!({
            "type": "attachment",
            "attachment_id": "attachment-id",
            "upload_url": "https://upload.example.com",
            "view_url": "https://view.example.com",
            "created_at": "2026-05-16T12:00:00Z"
        });

        let attachment: Attachment = serde_json::from_value(value).unwrap();
        assert_eq!(attachment.attachment_type, "attachment");
        assert_eq!(attachment.attachment_id, "attachment-id");
        assert_eq!(
            attachment.upload_url.as_deref(),
            Some("https://upload.example.com")
        );
        assert_eq!(
            attachment.view_url.as_deref(),
            Some("https://view.example.com")
        );
        assert_eq!(
            attachment.created_at.as_deref(),
            Some("2026-05-16T12:00:00Z")
        );
    }

    #[test]
    fn test_parse_attachment_response_errors_on_missing_data() {
        let err = parse_attachment_response(br#"{"data":null}"#, "attachment").unwrap_err();
        assert!(err.to_string().contains("Data not found"));
    }
}
