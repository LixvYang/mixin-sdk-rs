use serde::{Deserialize, Serialize};

use crate::{
    auth::sign_authentication_token,
    error::Error,
    pin::encrypt_ed25519_pin,
    request::{ApiResponse, request},
    safe::SafeUser,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LegacyTransferRequest {
    pub asset_id: String,
    pub opponent_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pin: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LegacyRawMultisig {
    #[serde(default)]
    pub receivers: Vec<String>,
    pub threshold: u8,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LegacyRawTransactionRequest {
    pub asset_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opponent_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opponent_multisig: Option<LegacyRawMultisig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pin: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LegacyGhostInputRequest {
    #[serde(default)]
    pub receivers: Vec<String>,
    pub index: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LegacyGhostKeys {
    #[serde(default, rename = "type")]
    pub type_name: Option<String>,
    #[serde(default)]
    pub keys: Vec<String>,
    #[serde(default)]
    pub mask: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LegacySnapshot {
    pub snapshot_id: String,
    #[serde(default, rename = "type")]
    pub type_name: Option<String>,
    #[serde(default)]
    pub asset_id: Option<String>,
    #[serde(default)]
    pub amount: Option<String>,
    #[serde(default)]
    pub closing_balance: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub opening_balance: Option<String>,
    #[serde(default)]
    pub snapshot_at: Option<String>,
    #[serde(default)]
    pub snapshot_hash: Option<String>,
    #[serde(default)]
    pub transaction_hash: Option<String>,
    #[serde(default)]
    pub opponent_id: Option<String>,
    #[serde(default)]
    pub opponent_key: Option<String>,
    #[serde(default)]
    pub opponent_receivers: Vec<String>,
    #[serde(default)]
    pub opponent_threshold: Option<u8>,
    #[serde(default)]
    pub trace_id: Option<String>,
    #[serde(default)]
    pub memo: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub output_index: Option<i64>,
    #[serde(default)]
    pub sender: Option<String>,
}

pub fn encrypt_legacy_pin(pin_hex: &str, safe_user: &SafeUser) -> Result<String, Error> {
    encrypt_ed25519_pin(pin_hex, now_nanos()?, safe_user)
}

pub async fn fetch_transfer_by_trace(
    trace_id: &str,
    safe_user: &SafeUser,
) -> Result<LegacySnapshot, Error> {
    let path = format!("/transfers/trace/{trace_id}");
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;
    parse_snapshot(body).await
}

pub async fn create_transfer(
    request_body: &LegacyTransferRequest,
    safe_user: &SafeUser,
) -> Result<LegacySnapshot, Error> {
    let path = "/transfers";
    let data_str = serde_json::to_string(request_body)?;
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;
    parse_snapshot(body).await
}

pub async fn create_transfer_with_pin(
    pin_hex: &str,
    request_body: &LegacyTransferRequest,
    safe_user: &SafeUser,
) -> Result<LegacySnapshot, Error> {
    let mut request_body = request_body.clone();
    request_body.pin = Some(encrypt_legacy_pin(pin_hex, safe_user)?);
    create_transfer(&request_body, safe_user).await
}

pub async fn send_raw_transaction(
    request_body: &LegacyRawTransactionRequest,
    safe_user: &SafeUser,
) -> Result<LegacySnapshot, Error> {
    let path = "/transactions";
    let data_str = serde_json::to_string(request_body)?;
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;
    parse_snapshot(body).await
}

pub async fn send_raw_transaction_with_pin(
    pin_hex: &str,
    request_body: &LegacyRawTransactionRequest,
    safe_user: &SafeUser,
) -> Result<LegacySnapshot, Error> {
    let mut request_body = request_body.clone();
    request_body.pin = Some(encrypt_legacy_pin(pin_hex, safe_user)?);
    send_raw_transaction(&request_body, safe_user).await
}

pub async fn request_legacy_ghost_keys(
    requests: &[LegacyGhostInputRequest],
    safe_user: &SafeUser,
) -> Result<Vec<LegacyGhostKeys>, Error> {
    let path = "/outputs";
    let data_str = serde_json::to_string(requests)?;
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<Vec<LegacyGhostKeys>> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain legacy ghost key data".to_string())
    })
}

async fn parse_snapshot(body: Vec<u8>) -> Result<LegacySnapshot, Error> {
    let parsed: ApiResponse<LegacySnapshot> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain legacy snapshot data".to_string())
    })
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
    fn test_legacy_transfer_request_serialization() {
        let request = LegacyTransferRequest {
            asset_id: "asset-id".to_string(),
            opponent_id: "opponent-id".to_string(),
            amount: Some("1".to_string()),
            trace_id: None,
            memo: Some("memo".to_string()),
            pin: Some("encrypted-pin".to_string()),
        };
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["asset_id"], "asset-id");
        assert_eq!(value["opponent_id"], "opponent-id");
        assert_eq!(value["pin"], "encrypted-pin");
    }

    #[test]
    fn test_legacy_raw_transaction_request_serialization() {
        let request = LegacyRawTransactionRequest {
            asset_id: "asset-id".to_string(),
            opponent_multisig: Some(LegacyRawMultisig {
                receivers: vec!["receiver-id".to_string()],
                threshold: 1,
            }),
            ..Default::default()
        };
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["opponent_multisig"]["threshold"], 1);
        assert_eq!(value["opponent_multisig"]["receivers"][0], "receiver-id");
    }

    #[test]
    fn test_legacy_snapshot_deserialize_transfer() {
        let raw = r#"{
            "snapshot_id": "snapshot-id",
            "type": "transfer",
            "asset_id": "asset-id",
            "amount": "1",
            "opponent_id": "opponent-id",
            "trace_id": "trace-id",
            "memo": "memo"
        }"#;
        let snapshot: LegacySnapshot = serde_json::from_str(raw).expect("snapshot");
        assert_eq!(snapshot.snapshot_id, "snapshot-id");
        assert_eq!(snapshot.type_name.as_deref(), Some("transfer"));
        assert_eq!(snapshot.opponent_id.as_deref(), Some("opponent-id"));
    }
}
