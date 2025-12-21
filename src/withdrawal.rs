use serde::{Deserialize, Serialize};

use crate::{
    auth::sign_authentication_token,
    error::Error,
    pin::encrypt_ed25519_pin,
    request::{ApiResponse, request},
    safe::SafeUser,
    tip::{sign_tip_body, tip_body_for_withdrawal},
};

pub const MIXIN_FEE_USER_ID: &str = "674d6776-d600-4346-af46-58e77d8df185";

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct WithdrawalRequest {
    pub address_id: String,
    pub amount: String,
    pub trace_id: String,
    #[serde(default)]
    pub memo: Option<String>,
    #[serde(default)]
    pub pin_base64: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct WithdrawalView {
    pub withdrawal_id: Option<String>,
    pub request_id: Option<String>,
    pub asset_id: Option<String>,
    pub amount: Option<String>,
    pub fee: Option<String>,
    pub destination: Option<String>,
    pub tag: Option<String>,
    pub snapshot_id: Option<String>,
    pub state: Option<String>,
    pub created_at: Option<String>,
}

pub async fn create_withdrawal(
    address_id: &str,
    amount: &str,
    fee: &str,
    trace_id: &str,
    memo: Option<&str>,
    safe_user: &SafeUser,
) -> Result<WithdrawalView, Error> {
    let tip_body = tip_body_for_withdrawal(address_id, amount, fee, trace_id, memo.unwrap_or(""));
    let pin = sign_tip_body(&tip_body, &safe_user.spend_private_key, safe_user.is_spend_private_sum)?;
    let pin_base64 = encrypt_ed25519_pin(&pin, now_nanos()?, safe_user)?;

    let data = WithdrawalRequest {
        address_id: address_id.to_string(),
        amount: amount.to_string(),
        trace_id: trace_id.to_string(),
        memo: memo.map(|m| m.to_string()),
        pin_base64: Some(pin_base64),
    };
    let data_str = serde_json::to_string(&data)?;
    let path = "/withdrawals";
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<WithdrawalView> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain withdrawal data".to_string()))
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
    fn test_withdrawal_request_serialization() {
        let request = WithdrawalRequest {
            address_id: "address-id".to_string(),
            amount: "1".to_string(),
            trace_id: "trace-id".to_string(),
            memo: Some("memo".to_string()),
            pin_base64: Some("pin".to_string()),
        };
        let value: serde_json::Value = serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["address_id"], "address-id");
        assert_eq!(value["amount"], "1");
        assert_eq!(value["trace_id"], "trace-id");
        assert_eq!(value["memo"], "memo");
        assert_eq!(value["pin_base64"], "pin");
    }
}
