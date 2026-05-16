use serde::{Deserialize, Serialize};

use crate::{
    auth::sign_authentication_token,
    error::Error,
    request::{ApiResponse, request},
    safe::SafeUser,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpponentMultisig {
    #[serde(default)]
    pub receivers: Vec<String>,
    pub threshold: u8,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransferPaymentRequest {
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
pub struct RawPaymentRequest {
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
    pub opponent_multisig: Option<OpponentMultisig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum PaymentRequest {
    Transfer(TransferPaymentRequest),
    Raw(RawPaymentRequest),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Payment {
    #[serde(default, rename = "type")]
    pub type_name: Option<String>,
    pub asset_id: String,
    #[serde(default)]
    pub amount: Option<String>,
    #[serde(default)]
    pub receivers: Vec<String>,
    #[serde(default)]
    pub threshold: Option<u8>,
    #[serde(default)]
    pub memo: Option<String>,
    #[serde(default)]
    pub trace_id: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub code_id: Option<String>,
}

pub async fn create_payment(
    request_body: &PaymentRequest,
    safe_user: &SafeUser,
) -> Result<Payment, Error> {
    let path = "/payments";
    let data_str = serde_json::to_string(request_body)?;
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<Payment> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain payment data".to_string()))
}

pub async fn create_transfer_payment(
    request_body: &TransferPaymentRequest,
    safe_user: &SafeUser,
) -> Result<Payment, Error> {
    create_payment(&PaymentRequest::Transfer(request_body.clone()), safe_user).await
}

pub async fn create_raw_payment(
    request_body: &RawPaymentRequest,
    safe_user: &SafeUser,
) -> Result<Payment, Error> {
    create_payment(&PaymentRequest::Raw(request_body.clone()), safe_user).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_payment_request_serialization() {
        let request = PaymentRequest::Transfer(TransferPaymentRequest {
            asset_id: "asset-id".to_string(),
            opponent_id: "user-id".to_string(),
            amount: Some("1.23".to_string()),
            trace_id: None,
            memo: None,
            pin: None,
        });
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["asset_id"], "asset-id");
        assert_eq!(value["opponent_id"], "user-id");
        assert_eq!(value["amount"], "1.23");
        assert!(value.get("pin").is_none());
    }

    #[test]
    fn test_raw_payment_request_serialization() {
        let request = PaymentRequest::Raw(RawPaymentRequest {
            asset_id: "asset-id".to_string(),
            opponent_multisig: Some(OpponentMultisig {
                receivers: vec!["a".to_string(), "b".to_string()],
                threshold: 2,
            }),
            ..Default::default()
        });
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["asset_id"], "asset-id");
        assert_eq!(value["opponent_multisig"]["threshold"], 2);
        assert_eq!(value["opponent_multisig"]["receivers"][0], "a");
    }
}
