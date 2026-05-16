use serde::{Deserialize, Serialize};
use url::form_urlencoded;

use crate::{
    auth::sign_authentication_token,
    error::Error,
    models::{CollectibleOutput, CollectibleToken},
    pin::encrypt_ed25519_pin,
    request::{ApiResponse, request},
    safe::SafeUser,
    utils::hash_members,
};

pub const COLLECTIBLE_ACTION_SIGN: &str = "sign";
pub const COLLECTIBLE_ACTION_UNLOCK: &str = "unlock";
pub const COLLECTIBLE_ACTION_CANCEL: &str = "cancel";

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CollectibleOutputQuery {
    pub members: String,
    pub threshold: u8,
    pub state: Option<String>,
    pub offset: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CollectibleOutputsRequest {
    #[serde(default)]
    pub members: Vec<String>,
    pub threshold: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CollectibleCollection {
    #[serde(default, rename = "type")]
    pub type_name: Option<String>,
    pub collection_id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CollectibleTransaction {
    #[serde(default, rename = "type")]
    pub type_name: Option<String>,
    pub request_id: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub token_id: Option<String>,
    #[serde(default)]
    pub amount: Option<String>,
    #[serde(default)]
    pub senders: Vec<String>,
    #[serde(default)]
    pub senders_threshold: Option<i64>,
    #[serde(default)]
    pub receivers: Vec<String>,
    #[serde(default)]
    pub receivers_threshold: Option<i64>,
    #[serde(default)]
    pub signers: Option<serde_json::Value>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub transaction_hash: Option<String>,
    #[serde(default)]
    pub raw_transaction: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub code_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct CollectibleTransferRequest<'a> {
    action: &'a str,
    raw: &'a str,
}

#[derive(Debug, Serialize)]
struct CollectiblePinRequest<'a> {
    pin: &'a str,
}

pub async fn read_collectible_token(
    token_id: &str,
    safe_user: &SafeUser,
) -> Result<CollectibleToken, Error> {
    let path = format!("/collectibles/tokens/{token_id}");
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    let parsed: ApiResponse<CollectibleToken> = serde_json::from_slice(&body)?;
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain collectible token".to_string())
    })
}

pub async fn read_collectible_collection(
    collection_id: &str,
    safe_user: &SafeUser,
) -> Result<CollectibleCollection, Error> {
    let path = format!("/collectibles/collections/{collection_id}");
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    let parsed: ApiResponse<CollectibleCollection> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain collectible collection".to_string())
    })
}

pub async fn list_collectible_outputs(
    query: &CollectibleOutputQuery,
    safe_user: &SafeUser,
) -> Result<Vec<CollectibleOutput>, Error> {
    let path = collectible_outputs_path(query);
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    let parsed: ApiResponse<Vec<CollectibleOutput>> = serde_json::from_slice(&body)?;
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain collectible outputs".to_string())
    })
}

pub async fn list_collectible_outputs_for_members(
    request_body: &CollectibleOutputsRequest,
    safe_user: &SafeUser,
) -> Result<Vec<CollectibleOutput>, Error> {
    let query = CollectibleOutputQuery {
        members: hash_members(&request_body.members),
        threshold: request_body.threshold,
        state: request_body.state.clone(),
        offset: request_body.offset.clone(),
        limit: request_body.limit,
    };
    list_collectible_outputs(&query, safe_user).await
}

pub async fn create_collectible_transfer(
    action: &str,
    raw: &str,
    safe_user: &SafeUser,
) -> Result<CollectibleTransaction, Error> {
    let path = "/collectibles/requests";
    let data_str = serde_json::to_string(&CollectibleTransferRequest { action, raw })?;
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;
    parse_transaction(body).await
}

pub async fn sign_collectible_request(
    request_id: &str,
    pin_base64: &str,
    safe_user: &SafeUser,
) -> Result<CollectibleTransaction, Error> {
    mutate_collectible_request(request_id, COLLECTIBLE_ACTION_SIGN, pin_base64, safe_user).await
}

pub async fn sign_collectible_request_with_pin(
    request_id: &str,
    pin_hex: &str,
    safe_user: &SafeUser,
) -> Result<CollectibleTransaction, Error> {
    let pin_base64 = encrypt_ed25519_pin(pin_hex, now_nanos()?, safe_user)?;
    sign_collectible_request(request_id, &pin_base64, safe_user).await
}

pub async fn cancel_collectible_request(
    request_id: &str,
    pin_base64: &str,
    safe_user: &SafeUser,
) -> Result<CollectibleTransaction, Error> {
    mutate_collectible_request(request_id, COLLECTIBLE_ACTION_CANCEL, pin_base64, safe_user).await
}

pub async fn cancel_collectible_request_with_pin(
    request_id: &str,
    pin_hex: &str,
    safe_user: &SafeUser,
) -> Result<CollectibleTransaction, Error> {
    let pin_base64 = encrypt_ed25519_pin(pin_hex, now_nanos()?, safe_user)?;
    cancel_collectible_request(request_id, &pin_base64, safe_user).await
}

pub async fn unlock_collectible_request(
    request_id: &str,
    pin_base64: &str,
    safe_user: &SafeUser,
) -> Result<CollectibleTransaction, Error> {
    mutate_collectible_request(request_id, COLLECTIBLE_ACTION_UNLOCK, pin_base64, safe_user).await
}

pub async fn unlock_collectible_request_with_pin(
    request_id: &str,
    pin_hex: &str,
    safe_user: &SafeUser,
) -> Result<CollectibleTransaction, Error> {
    let pin_base64 = encrypt_ed25519_pin(pin_hex, now_nanos()?, safe_user)?;
    unlock_collectible_request(request_id, &pin_base64, safe_user).await
}

async fn mutate_collectible_request(
    request_id: &str,
    action: &str,
    pin_base64: &str,
    safe_user: &SafeUser,
) -> Result<CollectibleTransaction, Error> {
    let path = format!("/collectibles/requests/{request_id}/{action}");
    let data_str = serde_json::to_string(&CollectiblePinRequest { pin: pin_base64 })?;
    let token = sign_authentication_token("POST", &path, &data_str, safe_user)?;
    let body = request("POST", &path, data_str.as_bytes(), &token).await?;
    parse_transaction(body).await
}

async fn parse_transaction(body: Vec<u8>) -> Result<CollectibleTransaction, Error> {
    let parsed: ApiResponse<CollectibleTransaction> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain collectible transaction".to_string())
    })
}

fn collectible_outputs_path(query: &CollectibleOutputQuery) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    serializer.append_pair("members", &query.members);
    serializer.append_pair("threshold", &query.threshold.to_string());
    serializer.append_pair("limit", &query.limit.unwrap_or(100).to_string());
    if let Some(state) = &query.state
        && !state.is_empty()
    {
        serializer.append_pair("state", state);
    }
    if let Some(offset) = &query.offset
        && !offset.is_empty()
    {
        serializer.append_pair("offset", offset);
    }
    format!("/collectibles/outputs?{}", serializer.finish())
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
    fn test_collectible_output_query_serialization() {
        let query = CollectibleOutputQuery {
            members: "members".to_string(),
            threshold: 1,
            state: Some("unspent".to_string()),
            offset: Some("offset".to_string()),
            limit: Some(50),
        };
        let path = collectible_outputs_path(&query);
        assert!(path.contains("members=members"));
        assert!(path.contains("threshold=1"));
        assert!(path.contains("state=unspent"));
    }

    #[test]
    fn test_collectible_outputs_request_hashes_members() {
        let request = CollectibleOutputsRequest {
            members: vec![
                "965e5c6e-434c-3fa9-b780-c50f43cd955c".to_string(),
                "d1e9ec7e-199d-4578-91a0-a69d9a7ba048".to_string(),
            ],
            threshold: 2,
            ..Default::default()
        };
        assert_eq!(
            hash_members(&request.members),
            "6064ec68a229a7d2fe2be652d11477f21705a742e08b75564fd085650f1deaeb"
        );
    }

    #[test]
    fn test_collectible_transfer_request_serialization() {
        let request = CollectibleTransferRequest {
            action: COLLECTIBLE_ACTION_SIGN,
            raw: "raw",
        };
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["action"], COLLECTIBLE_ACTION_SIGN);
        assert_eq!(value["raw"], "raw");
    }

    #[test]
    fn test_collectible_pin_request_uses_node_field_name() {
        let request = CollectiblePinRequest { pin: "encrypted" };
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["pin"], "encrypted");
    }
}
