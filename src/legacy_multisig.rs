use serde::{Deserialize, Serialize};
use url::form_urlencoded;

use crate::{
    auth::sign_authentication_token,
    error::Error,
    pin::encrypt_ed25519_pin,
    request::{ApiResponse, request},
    safe::SafeUser,
    utils::hash_members,
};

pub const MULTISIG_ACTION_SIGN: &str = "sign";
pub const MULTISIG_ACTION_UNLOCK: &str = "unlock";

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LegacyMultisigQuery {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub members_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LegacyMultisigUtxo {
    #[serde(default, rename = "type")]
    pub type_name: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub utxo_id: Option<String>,
    #[serde(default)]
    pub asset_id: Option<String>,
    #[serde(default)]
    pub transaction_hash: Option<String>,
    #[serde(default)]
    pub output_index: Option<i64>,
    #[serde(default)]
    pub amount: Option<String>,
    #[serde(default)]
    pub threshold: Option<i64>,
    #[serde(default)]
    pub members: Vec<String>,
    #[serde(default)]
    pub memo: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub sender: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub signed_by: Option<String>,
    #[serde(default)]
    pub signed_tx: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LegacyMultisigRequest {
    #[serde(default, rename = "type")]
    pub type_name: Option<String>,
    pub request_id: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub asset_id: Option<String>,
    #[serde(default)]
    pub amount: Option<String>,
    #[serde(default)]
    pub threshold: Option<i64>,
    #[serde(default)]
    pub senders: Vec<String>,
    #[serde(default)]
    pub receivers: Vec<String>,
    #[serde(default)]
    pub signers: Vec<String>,
    #[serde(default)]
    pub memo: Option<String>,
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
struct LegacyMultisigCreateRequest<'a> {
    action: &'a str,
    raw: &'a str,
}

#[derive(Debug, Serialize)]
struct LegacyMultisigPinRequest<'a> {
    pin_base64: &'a str,
}

pub fn legacy_multisig_members_hash<T, I>(members: I) -> String
where
    I: IntoIterator<Item = T>,
    T: AsRef<str>,
{
    hash_members(members)
}

pub fn encrypt_legacy_multisig_pin(pin_hex: &str, safe_user: &SafeUser) -> Result<String, Error> {
    encrypt_ed25519_pin(pin_hex, now_nanos()?, safe_user)
}

pub async fn list_legacy_multisigs(
    limit: u32,
    offset: Option<&str>,
    safe_user: &SafeUser,
) -> Result<Vec<LegacyMultisigUtxo>, Error> {
    let path = legacy_multisigs_path(limit, offset);
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;
    parse_utxos(body).await
}

pub async fn list_multisig_outputs(
    query: &LegacyMultisigQuery,
    safe_user: &SafeUser,
) -> Result<Vec<LegacyMultisigUtxo>, Error> {
    let path = multisig_outputs_path(query);
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;
    parse_utxos(body).await
}

pub async fn list_multisig_outputs_for_members<T, I>(
    members: I,
    threshold: u8,
    state: Option<&str>,
    offset: Option<&str>,
    limit: Option<u32>,
    safe_user: &SafeUser,
) -> Result<Vec<LegacyMultisigUtxo>, Error>
where
    I: IntoIterator<Item = T>,
    T: AsRef<str>,
{
    let query = LegacyMultisigQuery {
        members_hash: Some(legacy_multisig_members_hash(members)),
        threshold: Some(threshold),
        state: state.map(ToString::to_string),
        offset: offset.map(ToString::to_string),
        limit,
        order: None,
    };
    list_multisig_outputs(&query, safe_user).await
}

pub async fn create_multisig(
    action: &str,
    raw: &str,
    safe_user: &SafeUser,
) -> Result<LegacyMultisigRequest, Error> {
    let path = "/multisigs/requests";
    let data_str = serde_json::to_string(&LegacyMultisigCreateRequest { action, raw })?;
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;
    parse_request(body).await
}

pub async fn sign_multisig(
    request_id: &str,
    pin_base64: &str,
    safe_user: &SafeUser,
) -> Result<LegacyMultisigRequest, Error> {
    mutate_multisig_with_pin(request_id, MULTISIG_ACTION_SIGN, pin_base64, safe_user).await
}

pub async fn sign_multisig_with_pin(
    request_id: &str,
    pin_hex: &str,
    safe_user: &SafeUser,
) -> Result<LegacyMultisigRequest, Error> {
    let pin_base64 = encrypt_legacy_multisig_pin(pin_hex, safe_user)?;
    sign_multisig(request_id, &pin_base64, safe_user).await
}

pub async fn unlock_multisig(
    request_id: &str,
    pin_base64: &str,
    safe_user: &SafeUser,
) -> Result<(), Error> {
    mutate_multisig_with_pin_no_data(request_id, MULTISIG_ACTION_UNLOCK, pin_base64, safe_user)
        .await
}

pub async fn unlock_multisig_with_pin(
    request_id: &str,
    pin_hex: &str,
    safe_user: &SafeUser,
) -> Result<(), Error> {
    let pin_base64 = encrypt_legacy_multisig_pin(pin_hex, safe_user)?;
    unlock_multisig(request_id, &pin_base64, safe_user).await
}

pub async fn cancel_multisig(request_id: &str, safe_user: &SafeUser) -> Result<(), Error> {
    let path = format!("/multisigs/requests/{request_id}/cancel");
    let token = sign_authentication_token("POST", &path, "", safe_user)?;
    let body = request("POST", &path, &[], &token).await?;

    let parsed: ApiResponse<serde_json::Value> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    Ok(())
}

async fn mutate_multisig_with_pin(
    request_id: &str,
    action: &str,
    pin_base64: &str,
    safe_user: &SafeUser,
) -> Result<LegacyMultisigRequest, Error> {
    let path = format!("/multisigs/requests/{request_id}/{action}");
    let data_str = serde_json::to_string(&LegacyMultisigPinRequest { pin_base64 })?;
    let token = sign_authentication_token("POST", &path, &data_str, safe_user)?;
    let body = request("POST", &path, data_str.as_bytes(), &token).await?;
    parse_request(body).await
}

async fn mutate_multisig_with_pin_no_data(
    request_id: &str,
    action: &str,
    pin_base64: &str,
    safe_user: &SafeUser,
) -> Result<(), Error> {
    let path = format!("/multisigs/requests/{request_id}/{action}");
    let data_str = serde_json::to_string(&LegacyMultisigPinRequest { pin_base64 })?;
    let token = sign_authentication_token("POST", &path, &data_str, safe_user)?;
    let body = request("POST", &path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<serde_json::Value> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    Ok(())
}

async fn parse_utxos(body: Vec<u8>) -> Result<Vec<LegacyMultisigUtxo>, Error> {
    let parsed: ApiResponse<Vec<LegacyMultisigUtxo>> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain legacy multisig output data".to_string())
    })
}

async fn parse_request(body: Vec<u8>) -> Result<LegacyMultisigRequest, Error> {
    let parsed: ApiResponse<LegacyMultisigRequest> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain legacy multisig request data".to_string())
    })
}

fn legacy_multisigs_path(limit: u32, offset: Option<&str>) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    serializer.append_pair("limit", &limit.to_string());
    if let Some(offset) = offset
        && !offset.is_empty()
    {
        serializer.append_pair("offset", offset);
    }
    format!("/multisigs?{}", serializer.finish())
}

fn multisig_outputs_path(query: &LegacyMultisigQuery) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    serializer.append_pair("limit", &query.limit.unwrap_or(100).to_string());
    if let Some(offset) = &query.offset
        && !offset.is_empty()
    {
        serializer.append_pair("offset", offset);
    }
    if let Some(members_hash) = &query.members_hash
        && !members_hash.is_empty()
    {
        serializer.append_pair("members", members_hash);
    }
    if let Some(threshold) = query.threshold {
        serializer.append_pair("threshold", &threshold.to_string());
    }
    if let Some(state) = &query.state
        && !state.is_empty()
    {
        serializer.append_pair("state", state);
    }
    if let Some(order) = &query.order
        && !order.is_empty()
    {
        serializer.append_pair("order", order);
    }
    format!("/multisigs/outputs?{}", serializer.finish())
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
    fn test_legacy_multisigs_path() {
        assert_eq!(
            legacy_multisigs_path(50, Some("offset")),
            "/multisigs?limit=50&offset=offset"
        );
    }

    #[test]
    fn test_multisig_outputs_path() {
        let query = LegacyMultisigQuery {
            members_hash: Some("hash".to_string()),
            threshold: Some(2),
            state: Some("unspent".to_string()),
            offset: Some("offset".to_string()),
            limit: Some(20),
            order: Some("updated".to_string()),
        };
        assert_eq!(
            multisig_outputs_path(&query),
            "/multisigs/outputs?limit=20&offset=offset&members=hash&threshold=2&state=unspent&order=updated"
        );
    }

    #[test]
    fn test_create_request_serialization() {
        let request = LegacyMultisigCreateRequest {
            action: MULTISIG_ACTION_SIGN,
            raw: "raw",
        };
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["action"], MULTISIG_ACTION_SIGN);
        assert_eq!(value["raw"], "raw");
    }

    #[test]
    fn test_pin_request_uses_go_field_name() {
        let request = LegacyMultisigPinRequest { pin_base64: "pin" };
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["pin_base64"], "pin");
    }
}
