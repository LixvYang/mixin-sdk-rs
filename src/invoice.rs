use serde::{Deserialize, Serialize};

use crate::{
    auth::sign_authentication_token,
    error::Error,
    models::Invoice,
    request::{ApiResponse, request},
    safe::SafeUser,
};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct InvoiceRequest {
    pub amount: String,
    pub asset_id: String,
    #[serde(default)]
    pub memo: Option<String>,
    #[serde(default)]
    pub trace_id: Option<String>,
}

pub async fn create_invoice(
    amount: &str,
    asset_id: &str,
    memo: Option<&str>,
    trace_id: Option<&str>,
    safe_user: &SafeUser,
) -> Result<Invoice, Error> {
    let data = InvoiceRequest {
        amount: amount.to_string(),
        asset_id: asset_id.to_string(),
        memo: memo.map(|m| m.to_string()),
        trace_id: trace_id.map(|t| t.to_string()),
    };
    let data_str = serde_json::to_string(&data)?;
    let path = "/invoices";
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<Invoice> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain invoice data".to_string()))
}

pub async fn read_invoice(invoice_id: &str, safe_user: &SafeUser) -> Result<Invoice, Error> {
    let path = format!("/invoices/{invoice_id}");
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    let parsed: ApiResponse<Invoice> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain invoice data".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invoice_request_serialization() {
        let request = InvoiceRequest {
            amount: "1".to_string(),
            asset_id: "asset-id".to_string(),
            memo: Some("memo".to_string()),
            trace_id: Some("trace".to_string()),
        };
        let value: serde_json::Value = serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["amount"], "1");
        assert_eq!(value["asset_id"], "asset-id");
        assert_eq!(value["memo"], "memo");
        assert_eq!(value["trace_id"], "trace");
    }
}
