use serde::{Deserialize, Deserializer, Serialize};

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
pub struct TransactionReceiver {
    #[serde(default)]
    pub members: Option<Vec<String>>,
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

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct TransactionView {
    #[serde(default, rename = "type")]
    pub type_name: Option<String>,
    pub request_id: Option<String>,
    pub transaction_hash: Option<String>,
    pub asset: Option<String>,
    pub amount: Option<String>,
    pub extra: Option<String>,
    pub user_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_receivers")]
    pub receivers: Option<Vec<TransactionReceiver>>,
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
    let requests = [TransactionRequest {
        request_id: request_id.to_string(),
        raw: raw.to_string(),
    }];
    one_transaction(verify_transactions(&requests, safe_user).await?)
}

pub async fn verify_transactions(
    requests: &[TransactionRequest],
    safe_user: &SafeUser,
) -> Result<Vec<TransactionView>, Error> {
    let path = "/safe/transaction/requests";
    let data_str = serde_json::to_string(requests)?;
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<Vec<TransactionView>> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain transaction data".to_string())
    })
}

fn deserialize_optional_receivers<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<TransactionReceiver>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    let Some(value) = value else {
        return Ok(None);
    };
    match value {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::String(value) if value.is_empty() => Ok(None),
        serde_json::Value::String(value) => Ok(Some(vec![TransactionReceiver {
            members: Some(vec![value]),
            ..Default::default()
        }])),
        serde_json::Value::Array(values) => {
            let mut receivers = Vec::with_capacity(values.len());
            for value in values {
                match value {
                    serde_json::Value::String(member) => {
                        receivers.push(TransactionReceiver {
                            members: Some(vec![member]),
                            ..Default::default()
                        });
                    }
                    serde_json::Value::Object(_) => {
                        receivers
                            .push(serde_json::from_value(value).map_err(serde::de::Error::custom)?);
                    }
                    other => {
                        return Err(serde::de::Error::custom(format!(
                            "invalid receiver value: {other}"
                        )));
                    }
                }
            }
            Ok(Some(receivers))
        }
        serde_json::Value::Object(_) => Ok(Some(vec![
            serde_json::from_value(value).map_err(serde::de::Error::custom)?,
        ])),
        other => Err(serde::de::Error::custom(format!(
            "expected receiver object or array, got {other}"
        ))),
    }
}

pub async fn submit_transaction(
    request_id: &str,
    signed_raw: &str,
    safe_user: &SafeUser,
) -> Result<TransactionView, Error> {
    let requests = [TransactionRequest {
        request_id: request_id.to_string(),
        raw: signed_raw.to_string(),
    }];
    one_transaction(send_transactions(&requests, safe_user).await?)
}

pub async fn send_transactions(
    requests: &[TransactionRequest],
    safe_user: &SafeUser,
) -> Result<Vec<TransactionView>, Error> {
    let path = "/safe/transactions";
    let data_str = serde_json::to_string(requests)?;
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<Vec<TransactionView>> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain transaction data".to_string())
    })
}

pub async fn get_transaction(
    request_id: &str,
    safe_user: &SafeUser,
) -> Result<TransactionView, Error> {
    let path = format!("/safe/transactions/{request_id}");
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    let parsed: ApiResponse<TransactionView> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain transaction data".to_string())
    })
}

fn one_transaction(mut transactions: Vec<TransactionView>) -> Result<TransactionView, Error> {
    if transactions.len() != 1 {
        return Err(Error::DataNotFound(format!(
            "expected one transaction, got {}",
            transactions.len()
        )));
    }
    Ok(transactions.remove(0))
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
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["request_id"], "request-id");
        assert_eq!(value["raw"], "raw");
    }

    #[test]
    fn test_signed_transaction_serialization() {
        let request = SignedTransactionRequest {
            request_id: "request-id".to_string(),
            signed_raw: "signed".to_string(),
        };
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["request_id"], "request-id");
        assert_eq!(value["signed_raw"], "signed");
    }

    #[test]
    fn test_transaction_request_batch_serialization() {
        let request = TransactionRequest {
            request_id: "request-id".to_string(),
            raw: "raw".to_string(),
        };
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&[request]).unwrap()).unwrap();
        assert_eq!(value[0]["request_id"], "request-id");
        assert_eq!(value[0]["raw"], "raw");
    }

    #[test]
    fn test_transaction_view_receivers_deserialize() {
        let raw = r#"{
            "type": "kernel_transaction_request",
            "request_id": "request-id",
            "user_id": "user-id",
            "receivers": [
                {
                    "members": ["member-id"],
                    "members_hash": "hash",
                    "threshold": 1
                },
                "legacy-member-id"
            ],
            "senders": ["sender-id"],
            "views": ["view"]
        }"#;
        let view: TransactionView = serde_json::from_str(raw).expect("view");
        assert_eq!(
            view.type_name.as_deref(),
            Some("kernel_transaction_request")
        );
        assert_eq!(view.user_id.as_deref(), Some("user-id"));
        let receivers = view.receivers.expect("receivers");
        assert_eq!(receivers.len(), 2);
        assert_eq!(
            receivers[0].members.as_deref(),
            Some(["member-id".to_string()].as_slice())
        );
        assert_eq!(
            receivers[1].members.as_deref(),
            Some(["legacy-member-id".to_string()].as_slice())
        );
    }
}
