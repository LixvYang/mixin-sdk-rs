use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct User {
    pub user_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub identity_number: Option<String>,
    #[serde(default)]
    pub has_safe: Option<bool>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub membership: Option<Membership>,
    #[serde(default)]
    pub app_id: Option<String>,
    #[serde(default)]
    pub is_verified: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Membership {
    #[serde(default)]
    pub plan: Option<String>,
    #[serde(default)]
    pub expired_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Asset {
    pub asset_id: String,
    #[serde(default)]
    pub chain_id: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub balance: Option<String>,
    #[serde(default)]
    pub destination: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub price_usd: Option<String>,
    #[serde(default)]
    pub price_btc: Option<String>,
    #[serde(default)]
    pub change_usd: Option<String>,
    #[serde(default)]
    pub change_btc: Option<String>,
    #[serde(default)]
    pub confirmations: Option<i64>,
    #[serde(default)]
    pub dust: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub asset_key: Option<String>,
    #[serde(default)]
    pub display_symbol: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Output {
    #[serde(rename = "type")]
    pub output_type: Option<String>,
    pub output_id: String,
    #[serde(default)]
    pub transaction_hash: Option<String>,
    #[serde(default)]
    pub output_index: Option<u32>,
    #[serde(default)]
    pub asset_id: Option<String>,
    #[serde(default)]
    pub kernel_asset_id: Option<String>,
    #[serde(default)]
    pub amount: Option<String>,
    #[serde(default)]
    pub mask: Option<String>,
    #[serde(default)]
    pub keys: Option<Vec<String>>,
    #[serde(default)]
    pub senders_hash: Option<String>,
    #[serde(default)]
    pub senders_threshold: Option<i64>,
    #[serde(default)]
    pub senders: Option<Vec<String>>,
    #[serde(default)]
    pub receivers_hash: Option<String>,
    #[serde(default)]
    pub receivers_threshold: Option<i64>,
    #[serde(default)]
    pub receivers: Option<Vec<String>>,
    #[serde(default)]
    pub extra: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub sequence: Option<i64>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub signed_by: Option<Vec<String>>,
    #[serde(default)]
    pub signed_tx: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Snapshot {
    pub snapshot_id: String,
    #[serde(default, rename = "type")]
    pub type_name: Option<String>,
    #[serde(default)]
    pub asset_id: Option<String>,
    #[serde(default)]
    pub amount: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub data: Option<String>,
    #[serde(default)]
    pub trace_id: Option<String>,
    #[serde(default)]
    pub opponent_id: Option<String>,
    #[serde(default)]
    pub memo: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Invoice {
    pub invoice_id: String,
    #[serde(default)]
    pub payment_code: Option<String>,
    #[serde(default)]
    pub amount: Option<String>,
    #[serde(default)]
    pub memo: Option<String>,
    #[serde(default)]
    pub asset_id: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct CollectibleToken {
    pub token_id: String,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub mixin_id: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub nfo: Option<CollectibleNfo>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct CollectibleNfo {
    #[serde(default)]
    pub collection: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub media_url: Option<String>,
    #[serde(default)]
    pub mime: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct CollectibleOutput {
    pub output_id: String,
    #[serde(default)]
    pub token_id: Option<String>,
    #[serde(default)]
    pub transaction_hash: Option<String>,
    #[serde(default)]
    pub output_index: Option<u32>,
    #[serde(default)]
    pub receivers: Option<Vec<String>>,
    #[serde(default)]
    pub threshold: Option<i64>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub signed_by: Option<Vec<String>>,
    #[serde(default)]
    pub signed_tx: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_deserialize() {
        let raw = r#"{
            "user_id": "user-id",
            "session_id": "session-id",
            "identity_number": "7000123456",
            "has_safe": true,
            "full_name": "Alice",
            "avatar_url": "https://example.com/a.png",
            "created_at": "2024-01-02T03:04:05.000Z",
            "membership": {"plan": "PRO", "expired_at": "2025-01-01T00:00:00.000Z"}
        }"#;

        let user: User = serde_json::from_str(raw).expect("user");
        assert_eq!(user.user_id, "user-id");
        assert_eq!(user.session_id.as_deref(), Some("session-id"));
        assert_eq!(user.identity_number.as_deref(), Some("7000123456"));
        assert_eq!(user.has_safe, Some(true));
        assert_eq!(user.membership.unwrap().plan.as_deref(), Some("PRO"));
    }

    #[test]
    fn test_asset_deserialize() {
        let raw = r#"{
            "asset_id": "asset-id",
            "chain_id": "chain-id",
            "symbol": "BTC",
            "name": "Bitcoin",
            "balance": "1.2345",
            "destination": "bc1xxx",
            "tag": "",
            "price_usd": "50000",
            "price_btc": "1",
            "change_usd": "0.01",
            "change_btc": "0.01",
            "confirmations": 6,
            "dust": "0.0001",
            "updated_at": "2024-01-02T03:04:05.000Z",
            "icon_url": "https://example.com/btc.png",
            "asset_key": "btc-key",
            "display_symbol": "BTC",
            "display_name": "Bitcoin"
        }"#;

        let asset: Asset = serde_json::from_str(raw).expect("asset");
        assert_eq!(asset.asset_id, "asset-id");
        assert_eq!(asset.symbol.as_deref(), Some("BTC"));
        assert_eq!(asset.balance.as_deref(), Some("1.2345"));
        assert_eq!(asset.confirmations, Some(6));
    }

    #[test]
    fn test_output_deserialize() {
        let raw = r#"{
            "type": "transaction",
            "output_id": "output-id",
            "transaction_hash": "tx-hash",
            "output_index": 1,
            "asset_id": "asset-id",
            "kernel_asset_id": "kernel-asset-id",
            "amount": "0.5",
            "mask": "mask",
            "keys": ["key1", "key2"],
            "senders_hash": "senders-hash",
            "senders_threshold": 2,
            "senders": ["sender1", "sender2"],
            "receivers_hash": "receivers-hash",
            "receivers_threshold": 1,
            "receivers": ["receiver1"],
            "extra": "extra",
            "state": "unspent",
            "sequence": 123,
            "created_at": "2024-01-02T03:04:05.000Z",
            "updated_at": "2024-01-02T03:04:05.000Z",
            "signed_by": ["signer"],
            "signed_tx": "signed"
        }"#;

        let output: Output = serde_json::from_str(raw).expect("output");
        assert_eq!(output.output_id, "output-id");
        assert_eq!(output.output_type.as_deref(), Some("transaction"));
        assert_eq!(output.output_index, Some(1));
        assert_eq!(output.state.as_deref(), Some("unspent"));
    }

    #[test]
    fn test_snapshot_deserialize() {
        let raw = r#"{
            "snapshot_id": "snapshot-id",
            "type": "transfer",
            "asset_id": "asset-id",
            "amount": "1.23",
            "created_at": "2024-01-02T03:04:05.000Z",
            "trace_id": "trace-id",
            "opponent_id": "opponent-id",
            "memo": "hi"
        }"#;
        let snapshot: Snapshot = serde_json::from_str(raw).expect("snapshot");
        assert_eq!(snapshot.snapshot_id, "snapshot-id");
        assert_eq!(snapshot.asset_id.as_deref(), Some("asset-id"));
        assert_eq!(snapshot.amount.as_deref(), Some("1.23"));
    }

    #[test]
    fn test_invoice_deserialize() {
        let raw = r#"{
            "invoice_id": "invoice-id",
            "payment_code": "payment-code",
            "amount": "1",
            "memo": "memo",
            "asset_id": "asset-id",
            "created_at": "2024-01-02T03:04:05.000Z",
            "expires_at": "2024-01-02T04:04:05.000Z",
            "status": "paid"
        }"#;
        let invoice: Invoice = serde_json::from_str(raw).expect("invoice");
        assert_eq!(invoice.invoice_id, "invoice-id");
        assert_eq!(invoice.payment_code.as_deref(), Some("payment-code"));
        assert_eq!(invoice.status.as_deref(), Some("paid"));
    }

    #[test]
    fn test_collectible_token_deserialize() {
        let raw = r#"{
            "token_id": "token-id",
            "group": "group-id",
            "token": "token",
            "mixin_id": "mixin-id",
            "created_at": "2024-01-02T03:04:05.000Z",
            "nfo": {
                "collection": "collection",
                "name": "name",
                "description": "desc",
                "icon_url": "https://example.com/icon.png",
                "media_url": "https://example.com/media.png",
                "mime": "image/png"
            }
        }"#;
        let token: CollectibleToken = serde_json::from_str(raw).expect("token");
        assert_eq!(token.token_id, "token-id");
        assert_eq!(token.nfo.unwrap().mime.as_deref(), Some("image/png"));
    }

    #[test]
    fn test_collectible_output_deserialize() {
        let raw = r#"{
            "output_id": "output-id",
            "token_id": "token-id",
            "transaction_hash": "tx-hash",
            "output_index": 1,
            "receivers": ["r1"],
            "threshold": 1,
            "state": "unspent",
            "created_at": "2024-01-02T03:04:05.000Z",
            "updated_at": "2024-01-02T03:04:05.000Z",
            "signed_by": ["s1"],
            "signed_tx": "signed"
        }"#;
        let output: CollectibleOutput = serde_json::from_str(raw).expect("output");
        assert_eq!(output.output_id, "output-id");
        assert_eq!(output.state.as_deref(), Some("unspent"));
    }
}
