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

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct SafeSnapshotQuery {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opponent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct SafeSnapshotDeposit {
    #[serde(default)]
    pub deposit_hash: Option<String>,
    #[serde(default)]
    pub deposit_index: Option<i64>,
    #[serde(default)]
    pub sender: Option<String>,
    #[serde(default)]
    pub destination: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct SafeSnapshotWithdrawal {
    #[serde(default)]
    pub withdrawal_hash: Option<String>,
    #[serde(default)]
    pub receiver: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct SafeSnapshot {
    #[serde(default, rename = "type")]
    pub type_name: Option<String>,
    pub snapshot_id: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub opponent_id: Option<String>,
    #[serde(default)]
    pub transaction_hash: Option<String>,
    #[serde(default)]
    pub asset_id: Option<String>,
    #[serde(default)]
    pub kernel_asset_id: Option<String>,
    #[serde(default)]
    pub amount: Option<String>,
    #[serde(default)]
    pub memo: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub level: Option<i64>,
    #[serde(default)]
    pub trace_id: Option<String>,
    #[serde(default)]
    pub confirmations: Option<i64>,
    #[serde(default)]
    pub opening_balance: Option<String>,
    #[serde(default)]
    pub closing_balance: Option<String>,
    #[serde(default)]
    pub deposit: Option<SafeSnapshotDeposit>,
    #[serde(default)]
    pub withdrawal: Option<SafeSnapshotWithdrawal>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SafeSnapshotNotificationRequest {
    pub transaction_hash: String,
    pub output_index: i64,
    pub receiver_id: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct MessageWithSession {
    #[serde(default, rename = "type")]
    pub type_name: Option<String>,
    #[serde(default)]
    pub representative_id: Option<String>,
    #[serde(default)]
    pub quote_message_id: Option<String>,
    #[serde(default)]
    pub conversation_id: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    pub message_id: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub data: Option<String>,
    #[serde(default)]
    pub data_base64: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub silent: Option<bool>,
    #[serde(default)]
    pub expire_in: Option<i64>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
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

pub async fn list_safe_snapshots(
    query: &SafeSnapshotQuery,
    safe_user: &SafeUser,
) -> Result<Vec<SafeSnapshot>, Error> {
    let path = safe_snapshots_path(query);
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    let parsed: ApiResponse<Vec<SafeSnapshot>> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain safe snapshot data".to_string())
    })
}

pub async fn read_safe_snapshot(
    snapshot_id: &str,
    safe_user: &SafeUser,
) -> Result<SafeSnapshot, Error> {
    let path = format!("/safe/snapshots/{snapshot_id}");
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    let parsed: ApiResponse<SafeSnapshot> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain safe snapshot data".to_string())
    })
}

pub async fn notify_safe_snapshot(
    request_body: &SafeSnapshotNotificationRequest,
    safe_user: &SafeUser,
) -> Result<MessageWithSession, Error> {
    let path = "/safe/snapshots/notifications";
    let data_str = serde_json::to_string(request_body)?;
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<MessageWithSession> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed.data.ok_or_else(|| {
        Error::DataNotFound(
            "API response did not contain safe snapshot notification data".to_string(),
        )
    })
}

fn safe_snapshots_path(query: &SafeSnapshotQuery) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    serializer.append_pair("limit", &query.limit.unwrap_or(100).to_string());
    if let Some(app) = &query.app
        && !app.is_empty()
    {
        serializer.append_pair("app", app);
    }
    if let Some(asset) = &query.asset
        && !asset.is_empty()
    {
        serializer.append_pair("asset", asset);
    }
    if let Some(opponent) = &query.opponent
        && !opponent.is_empty()
    {
        serializer.append_pair("opponent", opponent);
    }
    if let Some(offset) = &query.offset
        && !offset.is_empty()
    {
        serializer.append_pair("offset", offset);
    }
    format!("/safe/snapshots?{}", serializer.finish())
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

    #[test]
    fn test_safe_snapshots_path() {
        let query = SafeSnapshotQuery {
            app: Some("app-id".to_string()),
            asset: Some("asset-id".to_string()),
            opponent: None,
            offset: Some("offset".to_string()),
            limit: Some(50),
        };
        assert_eq!(
            safe_snapshots_path(&query),
            "/safe/snapshots?limit=50&app=app-id&asset=asset-id&offset=offset"
        );
    }

    #[test]
    fn test_safe_snapshot_deserialize() {
        let raw = r#"{
            "type": "transfer",
            "snapshot_id": "snapshot-id",
            "asset_id": "asset-id",
            "amount": "1",
            "deposit": {"deposit_hash": "deposit-hash", "deposit_index": 1}
        }"#;
        let snapshot: SafeSnapshot = serde_json::from_str(raw).expect("safe snapshot");
        assert_eq!(snapshot.snapshot_id, "snapshot-id");
        assert_eq!(snapshot.deposit.unwrap().deposit_index, Some(1));
    }

    #[test]
    fn test_safe_snapshot_notification_serialization() {
        let request = SafeSnapshotNotificationRequest {
            transaction_hash: "hash".to_string(),
            output_index: 1,
            receiver_id: "receiver-id".to_string(),
        };
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["transaction_hash"], "hash");
        assert_eq!(value["output_index"], 1);
        assert_eq!(value["receiver_id"], "receiver-id");
    }
}
