use serde::{Deserialize, Serialize};
use url::form_urlencoded;

use crate::{
    auth::sign_authentication_token,
    error::Error,
    models::Snapshot,
    request::{ApiResponse, request},
    safe::SafeUser,
};

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct SnapshotQuery {
    pub offset: Option<String>,
    pub limit: Option<u32>,
    pub asset: Option<String>,
    pub r#type: Option<String>,
    pub opponent: Option<String>,
    pub trace: Option<String>,
    pub order: Option<String>,
}

pub async fn list_snapshots(
    query: &SnapshotQuery,
    safe_user: &SafeUser,
) -> Result<Vec<Snapshot>, Error> {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    if let Some(offset) = &query.offset {
        serializer.append_pair("offset", offset);
    }
    serializer.append_pair("limit", &query.limit.unwrap_or(100).to_string());
    if let Some(asset) = &query.asset {
        serializer.append_pair("asset", asset);
    }
    if let Some(snapshot_type) = &query.r#type {
        serializer.append_pair("type", snapshot_type);
    }
    if let Some(opponent) = &query.opponent {
        serializer.append_pair("opponent", opponent);
    }
    if let Some(trace) = &query.trace {
        serializer.append_pair("trace", trace);
    }
    if let Some(order) = &query.order {
        serializer.append_pair("order", order);
    }

    let query_str = serializer.finish();
    let path = format!("/snapshots?{query_str}");
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    let parsed: ApiResponse<Vec<Snapshot>> = serde_json::from_slice(&body)?;
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain snapshot data".to_string())
    })
}

pub async fn read_snapshot(snapshot_id: &str, safe_user: &SafeUser) -> Result<Snapshot, Error> {
    let path = format!("/snapshots/{snapshot_id}");
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    let parsed: ApiResponse<Snapshot> = serde_json::from_slice(&body)?;
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain snapshot data".to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_query_serialization() {
        let query = SnapshotQuery {
            offset: Some("offset".to_string()),
            limit: Some(50),
            asset: Some("asset-id".to_string()),
            r#type: Some("transfer".to_string()),
            opponent: None,
            trace: Some("trace".to_string()),
            order: Some("ASC".to_string()),
        };
        let mut serializer = form_urlencoded::Serializer::new(String::new());
        serializer.append_pair("offset", query.offset.as_ref().unwrap());
        serializer.append_pair("limit", &query.limit.unwrap().to_string());
        serializer.append_pair("asset", query.asset.as_ref().unwrap());
        serializer.append_pair("type", query.r#type.as_ref().unwrap());
        serializer.append_pair("trace", query.trace.as_ref().unwrap());
        serializer.append_pair("order", query.order.as_ref().unwrap());
        let query_str = serializer.finish();
        assert!(query_str.contains("offset=offset"));
        assert!(query_str.contains("limit=50"));
        assert!(query_str.contains("asset=asset-id"));
        assert!(query_str.contains("type=transfer"));
    }
}
