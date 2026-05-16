use serde::{Deserialize, Serialize};
use url::form_urlencoded;

use crate::{
    chain::NetworkChain,
    error::Error,
    request::{ApiResponse, request},
};

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq)]
pub struct NetworkInfo {
    #[serde(default, rename = "type")]
    pub type_name: Option<String>,
    #[serde(default)]
    pub assets: Vec<NetworkAsset>,
    #[serde(default)]
    pub chains: Vec<NetworkChain>,
    #[serde(default)]
    pub assets_count: Option<String>,
    #[serde(default)]
    pub peak_throughput: Option<String>,
    #[serde(default)]
    pub snapshots_count: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq)]
pub struct NetworkAsset {
    #[serde(default, rename = "type")]
    pub type_name: Option<String>,
    pub asset_id: String,
    #[serde(default)]
    pub chain_id: Option<String>,
    #[serde(default)]
    pub fee_asset_id: Option<String>,
    #[serde(default)]
    pub display_symbol: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub balance: Option<String>,
    #[serde(default)]
    pub destination: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub price_btc: Option<String>,
    #[serde(default)]
    pub price_usd: Option<String>,
    #[serde(default)]
    pub change_btc: Option<String>,
    #[serde(default)]
    pub change_usd: Option<String>,
    #[serde(default)]
    pub asset_key: Option<String>,
    #[serde(default)]
    pub precision: Option<i64>,
    #[serde(default)]
    pub mixin_id: Option<String>,
    #[serde(default)]
    pub kernel_asset_id: Option<String>,
    #[serde(default)]
    pub reserve: Option<String>,
    #[serde(default)]
    pub dust: Option<String>,
    #[serde(default)]
    pub confirmations: Option<i64>,
    #[serde(default)]
    pub capitalization: Option<f64>,
    #[serde(default)]
    pub liquidity: Option<String>,
    #[serde(default)]
    pub price_updated_at: Option<String>,
    #[serde(default)]
    pub withdrawal_memo_possibility: Option<String>,
    #[serde(default)]
    pub primitive_asset_id: Option<String>,
    #[serde(default)]
    pub level: Option<i64>,
    #[serde(default)]
    pub collection_hash: Option<String>,
    #[serde(default)]
    pub amount: Option<String>,
    #[serde(default)]
    pub fee: Option<String>,
    #[serde(default)]
    pub snapshots_count: Option<i64>,
    #[serde(default)]
    pub deposit_entries: Option<Vec<DepositEntry>>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq, Eq)]
pub struct DepositEntry {
    pub destination: String,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub properties: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq)]
pub struct AssetTicker {
    #[serde(default, rename = "type")]
    pub type_name: Option<String>,
    #[serde(default)]
    pub price_btc: Option<String>,
    #[serde(default)]
    pub price_usd: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq)]
pub struct NetworkSnapshotAsset {
    pub asset_id: String,
    #[serde(default)]
    pub chain_id: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default, rename = "type")]
    pub type_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq)]
pub struct NetworkSnapshot {
    #[serde(default, rename = "type")]
    pub type_name: Option<String>,
    #[serde(default)]
    pub amount: Option<String>,
    #[serde(default)]
    pub asset: Option<NetworkSnapshotAsset>,
    #[serde(default)]
    pub created_at: Option<String>,
    pub snapshot_id: String,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub snapshot_hash: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub trace_id: Option<String>,
    #[serde(default)]
    pub opponent_id: Option<String>,
    #[serde(default)]
    pub data: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NetworkSnapshotQuery {
    pub limit: Option<u32>,
    pub offset: Option<String>,
    pub asset: Option<String>,
    pub order: Option<String>,
}

pub async fn read_network_info() -> Result<NetworkInfo, Error> {
    let path = "/network";
    let body = request("GET", path, &[], "").await?;
    parse_data(&body, "network info")
}

pub async fn read_network_assets() -> Result<Vec<NetworkAsset>, Error> {
    Ok(read_network_info().await?.assets)
}

pub async fn read_network_assets_top(kind: Option<&str>) -> Result<Vec<NetworkAsset>, Error> {
    let path = path_with_kind("/network/assets/top", kind);
    let body = request("GET", &path, &[], "").await?;
    parse_data(&body, "network assets")
}

pub async fn read_network_asset(asset_id: &str) -> Result<NetworkAsset, Error> {
    let path = format!("/network/assets/{asset_id}");
    let body = request("GET", &path, &[], "").await?;
    parse_data(&body, "network asset")
}

pub async fn search_network_assets(
    keyword: &str,
    kind: Option<&str>,
) -> Result<Vec<NetworkAsset>, Error> {
    let path = path_with_kind(&format!("/network/assets/search/{keyword}"), kind);
    let body = request("GET", &path, &[], "").await?;
    parse_data(&body, "network assets")
}

pub async fn read_asset_ticker(asset_id: &str, offset: Option<&str>) -> Result<AssetTicker, Error> {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    serializer.append_pair("asset", asset_id);
    if let Some(offset) = offset
        && !offset.is_empty()
    {
        serializer.append_pair("offset", offset);
    }
    let path = format!("/network/ticker?{}", serializer.finish());
    let body = request("GET", &path, &[], "").await?;
    parse_data(&body, "asset ticker")
}

pub async fn read_network_snapshot(snapshot_id: &str) -> Result<NetworkSnapshot, Error> {
    let path = format!("/network/snapshots/{snapshot_id}");
    let body = request("GET", &path, &[], "").await?;
    parse_data(&body, "network snapshot")
}

pub async fn list_network_snapshots(
    query: &NetworkSnapshotQuery,
) -> Result<Vec<NetworkSnapshot>, Error> {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    serializer.append_pair("limit", &query.limit.unwrap_or(100).to_string());
    if let Some(offset) = &query.offset
        && !offset.is_empty()
    {
        serializer.append_pair("offset", offset);
    }
    if let Some(asset) = &query.asset
        && !asset.is_empty()
    {
        serializer.append_pair("asset", asset);
    }
    if let Some(order) = &query.order
        && (order == "ASC" || order == "DESC")
    {
        serializer.append_pair("order", order);
    }
    let path = format!("/network/snapshots?{}", serializer.finish());
    let body = request("GET", &path, &[], "").await?;
    parse_data(&body, "network snapshots")
}

fn path_with_kind(path: &str, kind: Option<&str>) -> String {
    let Some(kind) = kind else {
        return path.to_string();
    };
    if kind.is_empty() {
        return path.to_string();
    }
    let query = form_urlencoded::Serializer::new(String::new())
        .append_pair("kind", kind)
        .finish();
    format!("{path}?{query}")
}

fn parse_data<T>(body: &[u8], label: &str) -> Result<T, Error>
where
    T: for<'de> Deserialize<'de>,
{
    let parsed: ApiResponse<T> = serde_json::from_slice(body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound(label.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_info_deserialize() {
        let raw = r#"{
            "type": "network",
            "assets_count": "1",
            "assets": [{
                "type": "asset",
                "asset_id": "asset-id",
                "symbol": "BTC",
                "price_usd": "50000"
            }],
            "chains": [{
                "type": "chain",
                "chain_id": "chain-id",
                "name": "Bitcoin"
            }]
        }"#;
        let info: NetworkInfo = serde_json::from_str(raw).expect("network info");
        assert_eq!(info.assets_count.as_deref(), Some("1"));
        assert_eq!(info.assets[0].asset_id, "asset-id");
        assert_eq!(info.chains[0].chain_id, "chain-id");
    }

    #[test]
    fn test_network_snapshot_query_defaults_and_filters() {
        let query = NetworkSnapshotQuery {
            limit: None,
            offset: Some("offset".to_string()),
            asset: Some("asset-id".to_string()),
            order: Some("DESC".to_string()),
        };
        let mut serializer = form_urlencoded::Serializer::new(String::new());
        serializer.append_pair("limit", &query.limit.unwrap_or(100).to_string());
        serializer.append_pair("offset", query.offset.as_ref().unwrap());
        serializer.append_pair("asset", query.asset.as_ref().unwrap());
        serializer.append_pair("order", query.order.as_ref().unwrap());
        let query = serializer.finish();
        assert!(query.contains("limit=100"));
        assert!(query.contains("offset=offset"));
        assert!(query.contains("asset=asset-id"));
        assert!(query.contains("order=DESC"));
    }

    #[test]
    fn test_network_snapshot_deserialize() {
        let raw = r#"{
            "type": "snapshot",
            "snapshot_id": "snapshot-id",
            "amount": "1",
            "asset": {"asset_id": "asset-id", "symbol": "BTC"},
            "source": "TRANSFER",
            "state": "confirmed"
        }"#;
        let snapshot: NetworkSnapshot = serde_json::from_str(raw).expect("snapshot");
        assert_eq!(snapshot.snapshot_id, "snapshot-id");
        assert_eq!(snapshot.asset.unwrap().symbol.as_deref(), Some("BTC"));
    }
}
