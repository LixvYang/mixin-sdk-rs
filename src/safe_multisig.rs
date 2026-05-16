use serde::{Deserialize, Serialize};

use crate::{
    auth::sign_authentication_token,
    error::Error,
    request::{ApiResponse, request},
    safe::SafeUser,
    safe_transaction::{decode_safe_transaction, sign_safe_transaction_with_index},
    transaction::TransactionRequest,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SafeMultisigReceiver {
    #[serde(default)]
    pub members: Vec<String>,
    #[serde(default)]
    pub members_hash: Option<String>,
    #[serde(default)]
    pub threshold: Option<u8>,
    #[serde(default)]
    pub destination: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub withdrawal_hash: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SafeMultisigRequest {
    #[serde(default, rename = "type")]
    pub type_name: Option<String>,
    pub request_id: Option<String>,
    pub transaction_hash: Option<String>,
    pub asset_id: Option<String>,
    pub kernel_asset_id: Option<String>,
    pub amount: Option<String>,
    #[serde(default)]
    pub receivers: Vec<SafeMultisigReceiver>,
    #[serde(default)]
    pub senders: Vec<String>,
    pub senders_hash: Option<String>,
    pub senders_threshold: Option<i64>,
    #[serde(default)]
    pub signers: Vec<String>,
    #[serde(default)]
    pub revoked_by: Option<String>,
    pub extra: Option<String>,
    pub raw_transaction: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    #[serde(default)]
    pub inscription_hash: Option<String>,
    #[serde(default)]
    pub views: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SafeMultisigSignRequest<'a> {
    raw: &'a str,
}

pub async fn create_safe_multisig_requests(
    requests: &[TransactionRequest],
    safe_user: &SafeUser,
) -> Result<Vec<SafeMultisigRequest>, Error> {
    let path = "/safe/multisigs";
    let data_str = serde_json::to_string(requests)?;
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<Vec<SafeMultisigRequest>> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain safe multisig data".to_string())
    })
}

pub async fn create_safe_multisig_request(
    request_id: &str,
    raw: &str,
    safe_user: &SafeUser,
) -> Result<SafeMultisigRequest, Error> {
    let requests = [TransactionRequest {
        request_id: request_id.to_string(),
        raw: raw.to_string(),
    }];
    one_safe_multisig(create_safe_multisig_requests(&requests, safe_user).await?)
}

pub async fn fetch_safe_multisig_request(
    id_or_hash: &str,
    safe_user: &SafeUser,
) -> Result<SafeMultisigRequest, Error> {
    let path = format!("/safe/multisigs/{id_or_hash}");
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    let parsed: ApiResponse<SafeMultisigRequest> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain safe multisig data".to_string())
    })
}

pub async fn sign_safe_multisig_request(
    id_or_hash: &str,
    signed_raw: &str,
    safe_user: &SafeUser,
) -> Result<SafeMultisigRequest, Error> {
    let path = format!("/safe/multisigs/{id_or_hash}/sign");
    let data_str = serde_json::to_string(&SafeMultisigSignRequest { raw: signed_raw })?;
    let token = sign_authentication_token("POST", &path, &data_str, safe_user)?;
    let body = request("POST", &path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<SafeMultisigRequest> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain safe multisig data".to_string())
    })
}

pub async fn unlock_safe_multisig_request(
    id_or_hash: &str,
    safe_user: &SafeUser,
) -> Result<SafeMultisigRequest, Error> {
    mutate_empty_safe_multisig_request(id_or_hash, "unlock", safe_user).await
}

pub async fn cancel_safe_multisig_request(
    id_or_hash: &str,
    safe_user: &SafeUser,
) -> Result<SafeMultisigRequest, Error> {
    mutate_empty_safe_multisig_request(id_or_hash, "cancel", safe_user).await
}

pub fn safe_multisig_signer_index(senders: &[String], signer_user_id: &str) -> Result<u16, Error> {
    let mut sorted = senders.to_vec();
    sorted.sort();
    sorted
        .iter()
        .position(|sender| sender == signer_user_id)
        .map(|index| index as u16)
        .ok_or_else(|| {
            Error::Input(format!(
                "signer {signer_user_id} is not in safe multisig senders"
            ))
        })
}

pub fn sign_safe_multisig_raw(
    raw_transaction: &str,
    views: &[String],
    senders: &[String],
    signer_user_id: &str,
    spend_private_key: &str,
    is_spend_private_sum: bool,
) -> Result<String, Error> {
    let tx = decode_safe_transaction(raw_transaction)?;
    let signer_index = safe_multisig_signer_index(senders, signer_user_id)?;
    sign_safe_transaction_with_index(
        &tx,
        views,
        spend_private_key,
        is_spend_private_sum,
        signer_index,
    )
}

pub async fn fetch_sign_safe_multisig_request(
    id_or_hash: &str,
    safe_user: &SafeUser,
) -> Result<SafeMultisigRequest, Error> {
    let request = fetch_safe_multisig_request(id_or_hash, safe_user).await?;
    let raw = request.raw_transaction.as_ref().ok_or_else(|| {
        Error::DataNotFound("safe multisig response is missing raw_transaction".to_string())
    })?;
    let signed_raw = sign_safe_multisig_raw(
        raw,
        &request.views,
        &request.senders,
        &safe_user.user_id,
        &safe_user.spend_private_key,
        safe_user.is_spend_private_sum,
    )?;
    sign_safe_multisig_request(id_or_hash, &signed_raw, safe_user).await
}

async fn mutate_empty_safe_multisig_request(
    id_or_hash: &str,
    action: &str,
    safe_user: &SafeUser,
) -> Result<SafeMultisigRequest, Error> {
    let path = format!("/safe/multisigs/{id_or_hash}/{action}");
    let token = sign_authentication_token("POST", &path, "", safe_user)?;
    let body = request("POST", &path, &[], &token).await?;

    let parsed: ApiResponse<SafeMultisigRequest> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain safe multisig data".to_string())
    })
}

fn one_safe_multisig(mut requests: Vec<SafeMultisigRequest>) -> Result<SafeMultisigRequest, Error> {
    if requests.len() != 1 {
        return Err(Error::DataNotFound(format!(
            "expected one safe multisig request, got {}",
            requests.len()
        )));
    }
    Ok(requests.remove(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_multisig_sign_request_serialization() {
        let value: serde_json::Value = serde_json::from_str(
            &serde_json::to_string(&SafeMultisigSignRequest { raw: "raw" }).unwrap(),
        )
        .unwrap();
        assert_eq!(value["raw"], "raw");
    }

    #[test]
    fn test_safe_multisig_response_deserialize() {
        let raw = r#"{
            "type": "transaction_request",
            "request_id": "request-id",
            "transaction_hash": "tx-hash",
            "asset_id": "asset-id",
            "kernel_asset_id": "kernel-asset-id",
            "amount": "1",
            "receivers": [
                {
                    "members": ["receiver-id"],
                    "members_hash": "hash",
                    "threshold": 1
                }
            ],
            "senders": ["sender-b", "sender-a"],
            "senders_hash": "senders-hash",
            "senders_threshold": 2,
            "signers": ["sender-a"],
            "revoked_by": "",
            "extra": "extra",
            "raw_transaction": "raw",
            "created_at": "2026-05-16T12:55:48.71517Z",
            "updated_at": "2026-05-16T12:55:48.71517Z",
            "views": ["view"]
        }"#;
        let request: SafeMultisigRequest = serde_json::from_str(raw).expect("request");
        assert_eq!(request.request_id.as_deref(), Some("request-id"));
        assert_eq!(request.receivers.len(), 1);
        assert_eq!(request.senders.len(), 2);
        assert_eq!(request.views, vec!["view".to_string()]);
    }

    #[test]
    fn test_safe_multisig_signer_index_sorts_senders() {
        let senders = vec!["sender-b".to_string(), "sender-a".to_string()];
        assert_eq!(safe_multisig_signer_index(&senders, "sender-a").unwrap(), 0);
        assert_eq!(safe_multisig_signer_index(&senders, "sender-b").unwrap(), 1);
        assert!(safe_multisig_signer_index(&senders, "sender-c").is_err());
    }
}
