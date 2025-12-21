use serde::{Deserialize, Serialize};

use crate::{
    auth::sign_authentication_token,
    error::Error,
    request::{ApiResponse, request},
    safe::SafeUser,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionRequest {
    pub request_id: String,
    pub raw: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignedTransactionRequest {
    pub request_id: String,
    #[serde(rename = "signed_raw")]
    pub signed_raw: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct TransactionView {
    pub request_id: Option<String>,
    pub transaction_hash: Option<String>,
    pub asset: Option<String>,
    pub amount: Option<String>,
    pub extra: Option<String>,
    pub senders: Option<Vec<String>>,
    pub senders_hash: Option<String>,
    pub senders_threshold: Option<i64>,
    pub signers: Option<Vec<String>>,
    pub state: Option<String>,
    pub raw_transaction: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub snapshot_id: Option<String>,
    pub snapshot_hash: Option<String>,
    pub snapshot_at: Option<String>,
    pub views: Option<Vec<String>>,
}

pub async fn create_transaction_request(
    request_id: &str,
    raw: &str,
    safe_user: &SafeUser,
) -> Result<TransactionView, Error> {
    let path = "/safe/transaction/requests";
    let data = TransactionRequest {
        request_id: request_id.to_string(),
        raw: raw.to_string(),
    };
    let data_str = serde_json::to_string(&data)?;
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<TransactionView> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain transaction data".to_string()))
}

pub async fn submit_transaction(
    request_id: &str,
    signed_raw: &str,
    safe_user: &SafeUser,
) -> Result<TransactionView, Error> {
    let path = "/safe/transactions";
    let data = SignedTransactionRequest {
        request_id: request_id.to_string(),
        signed_raw: signed_raw.to_string(),
    };
    let data_str = serde_json::to_string(&data)?;
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<TransactionView> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain transaction data".to_string()))
}

pub async fn get_transaction(request_id: &str, safe_user: &SafeUser) -> Result<TransactionView, Error> {
    let path = format!("/safe/transactions/{request_id}");
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    let parsed: ApiResponse<TransactionView> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain transaction data".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_request_serialization() {
        let request = TransactionRequest {
            request_id: "request-id".to_string(),
            raw: "raw".to_string(),
        };
        let value: serde_json::Value = serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["request_id"], "request-id");
        assert_eq!(value["raw"], "raw");
    }

    #[test]
    fn test_signed_transaction_serialization() {
        let request = SignedTransactionRequest {
            request_id: "request-id".to_string(),
            signed_raw: "signed".to_string(),
        };
        let value: serde_json::Value = serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["request_id"], "request-id");
        assert_eq!(value["signed_raw"], "signed");
    }
}
