use serde::{Deserialize, Serialize};
use url::form_urlencoded;

use crate::{
    error::Error,
    request::{ApiResponse, request},
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExternalTransactionQuery {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExternalTransaction {
    pub transaction_id: String,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub transaction_hash: Option<String>,
    #[serde(default)]
    pub sender: Option<String>,
    #[serde(default)]
    pub chain_id: Option<String>,
    #[serde(default)]
    pub asset_id: Option<String>,
    #[serde(default)]
    pub amount: Option<String>,
    #[serde(default)]
    pub destination: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub confirmations: Option<String>,
    #[serde(default)]
    pub threshold: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ExternalProxyRequest {
    pub method: String,
    #[serde(default)]
    pub params: Vec<serde_json::Value>,
}

pub async fn external_transactions(
    query: &ExternalTransactionQuery,
) -> Result<Vec<ExternalTransaction>, Error> {
    let path = external_transactions_path(query);
    let body = request("GET", &path, &[], "").await?;

    let parsed: ApiResponse<Vec<ExternalTransaction>> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain external transaction data".to_string())
    })
}

pub async fn external_proxy(
    request_body: &ExternalProxyRequest,
) -> Result<serde_json::Value, Error> {
    let path = "/external/proxy";
    let data_str = serde_json::to_string(request_body)?;
    let body = request("POST", path, data_str.as_bytes(), "").await?;

    let parsed: ApiResponse<serde_json::Value> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain external proxy data".to_string())
    })
}

fn external_transactions_path(query: &ExternalTransactionQuery) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    if let Some(asset) = &query.asset
        && !asset.is_empty()
    {
        serializer.append_pair("asset", asset);
    }
    if let Some(destination) = &query.destination
        && !destination.is_empty()
    {
        serializer.append_pair("destination", destination);
    }
    if let Some(tag) = &query.tag
        && !tag.is_empty()
    {
        serializer.append_pair("tag", tag);
    }
    if let Some(order) = &query.order
        && !order.is_empty()
    {
        serializer.append_pair("order", order);
    }
    if let Some(offset) = &query.offset
        && !offset.is_empty()
    {
        serializer.append_pair("offset", offset);
    }
    if let Some(limit) = query.limit {
        serializer.append_pair("limit", &limit.to_string());
    }
    if let Some(user) = &query.user
        && !user.is_empty()
    {
        serializer.append_pair("user", user);
    }
    let query = serializer.finish();
    if query.is_empty() {
        "/external/transactions".to_string()
    } else {
        format!("/external/transactions?{query}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_external_transactions_path() {
        let query = ExternalTransactionQuery {
            asset: Some("asset-id".to_string()),
            order: Some("DESC".to_string()),
            limit: Some(100),
            ..Default::default()
        };
        assert_eq!(
            external_transactions_path(&query),
            "/external/transactions?asset=asset-id&order=DESC&limit=100"
        );
    }

    #[test]
    fn test_external_proxy_request_serialization() {
        let request = ExternalProxyRequest {
            method: "sendrawtransaction".to_string(),
            params: vec![serde_json::json!("raw")],
        };
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["method"], "sendrawtransaction");
        assert_eq!(value["params"][0], "raw");
    }
}
