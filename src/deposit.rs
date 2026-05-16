use serde::{Deserialize, Serialize};
use url::form_urlencoded;

use crate::{
    auth::sign_authentication_token,
    error::Error,
    request::{ApiResponse, request},
    safe::SafeUser,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DepositEntryRequest {
    pub chain_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DepositEntry {
    #[serde(default, rename = "type")]
    pub type_name: Option<String>,
    pub entry_id: String,
    #[serde(default)]
    pub threshold: Option<i64>,
    #[serde(default)]
    pub members: Vec<String>,
    #[serde(default)]
    pub destination: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub signature: Option<String>,
    #[serde(default)]
    pub chain_id: Option<String>,
    #[serde(default)]
    pub is_primary: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SafePendingDepositQuery {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SafePendingDeposit {
    #[serde(default)]
    pub amount: Option<String>,
    #[serde(default)]
    pub asset_id: Option<String>,
    #[serde(default)]
    pub asset_key: Option<String>,
    #[serde(default)]
    pub block_hash: Option<String>,
    #[serde(default)]
    pub block_number: Option<i64>,
    #[serde(default)]
    pub chain_id: Option<String>,
    #[serde(default)]
    pub confirmations: Option<i64>,
    #[serde(default)]
    pub created_at: Option<String>,
    pub deposit_id: String,
    #[serde(default)]
    pub destination: Option<String>,
    #[serde(default)]
    pub extra: Option<String>,
    #[serde(default)]
    pub kernel_asset_id: Option<String>,
    #[serde(default)]
    pub output_index: Option<i64>,
    #[serde(default)]
    pub sender: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub threshold: Option<i64>,
    #[serde(default)]
    pub transaction_hash: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

pub async fn create_deposit_entries(
    request_body: &DepositEntryRequest,
    safe_user: &SafeUser,
) -> Result<Vec<DepositEntry>, Error> {
    let path = "/safe/deposit/entries";
    let data_str = serde_json::to_string(request_body)?;
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<Vec<DepositEntry>> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain deposit entry data".to_string())
    })
}

pub async fn create_deposit_entry(
    chain_id: &str,
    members: &[String],
    threshold: i64,
    safe_user: &SafeUser,
) -> Result<Vec<DepositEntry>, Error> {
    create_deposit_entries(
        &DepositEntryRequest {
            chain_id: chain_id.to_string(),
            members: Some(members.to_vec()),
            threshold: Some(threshold),
        },
        safe_user,
    )
    .await
}

pub async fn create_primary_deposit_entry(
    chain_id: &str,
    safe_user: &SafeUser,
) -> Result<Vec<DepositEntry>, Error> {
    create_deposit_entries(
        &DepositEntryRequest {
            chain_id: chain_id.to_string(),
            members: None,
            threshold: None,
        },
        safe_user,
    )
    .await
}

pub async fn fetch_pending_safe_deposits(
    query: &SafePendingDepositQuery,
) -> Result<Vec<SafePendingDeposit>, Error> {
    let path = pending_deposits_path(query);
    let body = request("GET", &path, &[], "").await?;

    let parsed: ApiResponse<Vec<SafePendingDeposit>> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain pending safe deposit data".to_string())
    })
}

fn pending_deposits_path(query: &SafePendingDepositQuery) -> String {
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
    if let Some(offset) = &query.offset
        && !offset.is_empty()
    {
        serializer.append_pair("offset", offset);
    }
    if let Some(limit) = query.limit {
        serializer.append_pair("limit", &limit.to_string());
    }
    let query = serializer.finish();
    if query.is_empty() {
        "/safe/deposits".to_string()
    } else {
        format!("/safe/deposits?{query}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deposit_entry_request_omits_primary_options() {
        let request = DepositEntryRequest {
            chain_id: "chain-id".to_string(),
            members: None,
            threshold: None,
        };
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["chain_id"], "chain-id");
        assert!(value.get("members").is_none());
        assert!(value.get("threshold").is_none());
    }

    #[test]
    fn test_pending_deposits_path() {
        let query = SafePendingDepositQuery {
            asset: Some("asset-id".to_string()),
            destination: Some("dest".to_string()),
            tag: None,
            offset: Some("offset".to_string()),
            limit: Some(20),
        };
        assert_eq!(
            pending_deposits_path(&query),
            "/safe/deposits?asset=asset-id&destination=dest&offset=offset&limit=20"
        );
    }
}
