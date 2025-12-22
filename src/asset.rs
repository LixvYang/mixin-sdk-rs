use serde::{Deserialize, Serialize};
use url::form_urlencoded;

use crate::{
    auth::sign_authentication_token,
    error::Error,
    models::Asset,
    request::{ApiResponse, request},
    safe::SafeUser,
};

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct AssetFee {
    #[serde(rename = "type")]
    pub fee_type: Option<String>,
    pub asset_id: Option<String>,
    pub amount: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct AssetNetwork {
    pub asset_id: Option<String>,
    pub chain_id: Option<String>,
    pub asset_key: Option<String>,
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub icon_url: Option<String>,
    pub price_usd: Option<String>,
    pub price_btc: Option<String>,
    pub change_usd: Option<String>,
    pub change_btc: Option<String>,
    pub confirmations: Option<i64>,
    pub balance: Option<String>,
}

pub async fn list_assets(safe_user: &SafeUser) -> Result<Vec<Asset>, Error> {
    let path = "/assets";
    let token = sign_authentication_token("GET", path, "", safe_user)?;
    let body = request("GET", path, &[], &token).await?;

    let parsed: ApiResponse<Vec<Asset>> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain asset data".to_string()))
}

pub async fn read_asset(asset_id: &str, safe_user: &SafeUser) -> Result<Asset, Error> {
    let path = format!("/assets/{asset_id}");
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    let parsed: ApiResponse<Asset> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain asset data".to_string()))
}

pub async fn fetch_assets(asset_ids: &[String], safe_user: &SafeUser) -> Result<Vec<Asset>, Error> {
    let data_str = serde_json::to_string(asset_ids)?;
    let path = "/safe/assets/fetch";
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<Vec<Asset>> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain asset data".to_string()))
}

pub async fn read_asset_fees(
    asset_id: &str,
    destination: &str,
    safe_user: &SafeUser,
) -> Result<Vec<AssetFee>, Error> {
    let query = form_urlencoded::Serializer::new(String::new())
        .append_pair("destination", destination)
        .finish();
    let path = format!("/safe/assets/{asset_id}/fees?{query}");
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    let parsed: ApiResponse<Vec<AssetFee>> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain fee data".to_string()))
}

pub async fn read_network_assets() -> Result<Vec<AssetNetwork>, Error> {
    let path = "/network";
    let body = request("GET", path, &[], "").await?;

    let parsed: ApiResponse<Vec<AssetNetwork>> = serde_json::from_slice(&body)?;
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain network assets".to_string())
    })
}

pub async fn read_network_assets_top() -> Result<Vec<AssetNetwork>, Error> {
    let path = "/network/assets/top";
    let body = request("GET", path, &[], "").await?;

    let parsed: ApiResponse<Vec<AssetNetwork>> = serde_json::from_slice(&body)?;
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain network assets".to_string())
    })
}

pub async fn read_network_asset(asset_id: &str) -> Result<AssetNetwork, Error> {
    let path = format!("/network/assets/{asset_id}");
    let body = request("GET", &path, &[], "").await?;

    let parsed: ApiResponse<AssetNetwork> = serde_json::from_slice(&body)?;
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain network asset".to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_fee_deserialize() {
        let raw = r#"{
            "type": "fee",
            "asset_id": "asset-id",
            "amount": "0.01"
        }"#;
        let fee: AssetFee = serde_json::from_str(raw).expect("fee");
        assert_eq!(fee.asset_id.as_deref(), Some("asset-id"));
        assert_eq!(fee.amount.as_deref(), Some("0.01"));
    }

    #[test]
    fn test_asset_network_deserialize() {
        let raw = r#"{
            "asset_id": "asset-id",
            "chain_id": "chain-id",
            "symbol": "BTC",
            "name": "Bitcoin",
            "icon_url": "https://example.com/btc.png",
            "price_usd": "50000",
            "price_btc": "1",
            "change_usd": "0.01",
            "change_btc": "0.01",
            "confirmations": 6,
            "balance": "0"
        }"#;
        let asset: AssetNetwork = serde_json::from_str(raw).expect("asset");
        assert_eq!(asset.asset_id.as_deref(), Some("asset-id"));
        assert_eq!(asset.symbol.as_deref(), Some("BTC"));
        assert_eq!(asset.confirmations, Some(6));
    }
}
