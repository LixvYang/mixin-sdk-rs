use serde::{Deserialize, Serialize};

use crate::{
    error::Error,
    request::{ApiResponse, request},
};

pub const INSCRIPTION_MODE_INSTANT: u8 = 1;
pub const INSCRIPTION_MODE_DONE: u8 = 2;
pub const INSCRIPTION_OPERATION_DEPLOY: &str = "deploy";
pub const INSCRIPTION_OPERATION_INSCRIBE: &str = "inscribe";
pub const INSCRIPTION_OPERATION_DISTRIBUTE: &str = "distribute";
pub const INSCRIPTION_OPERATION_OCCUPY: &str = "occupy";

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq, Eq)]
pub struct InscriptionTreasury {
    pub ratio: String,
    pub recipient: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct InscriptionDeploy {
    pub version: u8,
    #[serde(default = "default_deploy_operation")]
    pub operation: String,
    pub mode: u8,
    pub unit: String,
    pub supply: String,
    pub symbol: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub icon: String,
    /// Legacy Go SDK field. Current inscription spec uses `validation`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub treasury: Option<InscriptionTreasury>,
}

impl Default for InscriptionDeploy {
    fn default() -> Self {
        Self {
            version: 1,
            operation: INSCRIPTION_OPERATION_DEPLOY.to_string(),
            mode: 0,
            unit: String::new(),
            supply: String::new(),
            symbol: String::new(),
            name: String::new(),
            description: None,
            icon: String::new(),
            checksum: None,
            validation: None,
            treasury: None,
        }
    }
}

impl InscriptionDeploy {
    pub fn new(
        mode: u8,
        unit: impl Into<String>,
        supply: impl Into<String>,
        symbol: impl Into<String>,
        name: impl Into<String>,
        icon: impl Into<String>,
    ) -> Self {
        Self {
            mode,
            unit: unit.into(),
            supply: supply.into(),
            symbol: symbol.into(),
            name: name.into(),
            icon: icon.into(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct InscriptionInscribe {
    #[serde(default = "default_inscribe_operation")]
    pub operation: String,
    pub recipient: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

impl Default for InscriptionInscribe {
    fn default() -> Self {
        Self {
            operation: INSCRIPTION_OPERATION_INSCRIBE.to_string(),
            recipient: String::new(),
            content: None,
        }
    }
}

impl InscriptionInscribe {
    pub fn new(recipient: impl Into<String>, content: Option<String>) -> Self {
        Self {
            recipient: recipient.into(),
            content,
            ..Default::default()
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct InscriptionDistribute {
    #[serde(
        rename = "distribute",
        alias = "operation",
        default = "default_distribute_operation"
    )]
    pub operation: String,
    pub sequence: u64,
}

impl Default for InscriptionDistribute {
    fn default() -> Self {
        Self {
            operation: INSCRIPTION_OPERATION_DISTRIBUTE.to_string(),
            sequence: 0,
        }
    }
}

impl InscriptionDistribute {
    pub fn new(sequence: u64) -> Self {
        Self {
            sequence,
            ..Default::default()
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct InscriptionOccupy {
    #[serde(default = "default_occupy_operation")]
    pub operation: String,
    pub sequence: u64,
}

impl Default for InscriptionOccupy {
    fn default() -> Self {
        Self {
            operation: INSCRIPTION_OPERATION_OCCUPY.to_string(),
            sequence: 0,
        }
    }
}

impl InscriptionOccupy {
    pub fn new(sequence: u64) -> Self {
        Self {
            sequence,
            ..Default::default()
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq, Eq)]
pub struct Collection {
    pub asset_key: String,
    pub collection_hash: String,
    #[serde(default)]
    pub kernel_asset_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub supply: Option<String>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub minimum_price: Option<String>,
    #[serde(default)]
    pub treasury: Option<InscriptionTreasury>,
    #[serde(default, rename = "type")]
    pub type_name: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq, Eq)]
pub struct Inscription {
    pub inscription_hash: String,
    pub collection_hash: String,
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub content_url: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub occupied_by: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub recipient: Option<String>,
    #[serde(default)]
    pub sequence: Option<i64>,
    #[serde(default, rename = "type")]
    pub type_name: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

pub async fn read_collection(collection_hash: &str) -> Result<Collection, Error> {
    let path = format!("/safe/inscriptions/collections/{collection_hash}");
    let body = request("GET", &path, &[], "").await?;
    parse_data(&body, "collection")
}

pub async fn read_inscription(inscription_hash: &str) -> Result<Inscription, Error> {
    let path = format!("/safe/inscriptions/items/{inscription_hash}");
    let body = request("GET", &path, &[], "").await?;
    parse_data(&body, "inscription")
}

pub async fn read_collection_items(collection_hash: &str) -> Result<Vec<Inscription>, Error> {
    let path = format!("/safe/inscriptions/collections/{collection_hash}/items");
    let body = request("GET", &path, &[], "").await?;
    parse_data(&body, "collection items")
}

pub fn encode_inscription_extra<T>(operation: &T) -> Result<Vec<u8>, Error>
where
    T: Serialize,
{
    Ok(serde_json::to_vec(operation)?)
}

pub fn encode_inscription_deploy(deploy: &InscriptionDeploy) -> Result<Vec<u8>, Error> {
    encode_inscription_extra(deploy)
}

pub fn encode_inscription_inscribe(inscribe: &InscriptionInscribe) -> Result<Vec<u8>, Error> {
    encode_inscription_extra(inscribe)
}

pub fn encode_inscription_distribute(distribute: &InscriptionDistribute) -> Result<Vec<u8>, Error> {
    encode_inscription_extra(distribute)
}

pub fn encode_inscription_occupy(occupy: &InscriptionOccupy) -> Result<Vec<u8>, Error> {
    encode_inscription_extra(occupy)
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

fn default_deploy_operation() -> String {
    INSCRIPTION_OPERATION_DEPLOY.to_string()
}

fn default_inscribe_operation() -> String {
    INSCRIPTION_OPERATION_INSCRIBE.to_string()
}

fn default_distribute_operation() -> String {
    INSCRIPTION_OPERATION_DISTRIBUTE.to_string()
}

fn default_occupy_operation() -> String {
    INSCRIPTION_OPERATION_OCCUPY.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_deserialize() {
        let raw = r#"{
            "type": "collection",
            "asset_key": "asset-key",
            "collection_hash": "collection-hash",
            "name": "Collection",
            "symbol": "COL",
            "treasury": {"ratio": "0.1", "recipient": "MIX..."}
        }"#;
        let collection: Collection = serde_json::from_str(raw).expect("collection");
        assert_eq!(collection.collection_hash, "collection-hash");
        assert_eq!(collection.treasury.unwrap().ratio, "0.1");
    }

    #[test]
    fn test_inscription_deserialize() {
        let raw = r#"{
            "type": "inscription",
            "inscription_hash": "inscription-hash",
            "collection_hash": "collection-hash",
            "content_type": "text/plain",
            "sequence": 1
        }"#;
        let inscription: Inscription = serde_json::from_str(raw).expect("inscription");
        assert_eq!(inscription.inscription_hash, "inscription-hash");
        assert_eq!(inscription.sequence, Some(1));
    }

    #[test]
    fn test_encode_inscription_deploy_extra() {
        let mut deploy = InscriptionDeploy::new(
            INSCRIPTION_MODE_INSTANT,
            "1000000",
            "1000000000",
            "MAO",
            "Mao",
            "image/webp;base64,AAAA",
        );
        deploy.description = Some("demo".to_string());
        deploy.validation = Some("CHECKSUM:base64".to_string());

        let extra = encode_inscription_deploy(&deploy).expect("extra");
        let value: serde_json::Value = serde_json::from_slice(&extra).expect("json");

        assert_eq!(value["version"], 1);
        assert_eq!(value["operation"], INSCRIPTION_OPERATION_DEPLOY);
        assert_eq!(value["mode"], INSCRIPTION_MODE_INSTANT);
        assert_eq!(value["description"], "demo");
        assert_eq!(value["validation"], "CHECKSUM:base64");
    }

    #[test]
    fn test_encode_inscription_write_operations() {
        let inscribe =
            InscriptionInscribe::new("MIX-address", Some("text/plain;charset=UTF-8,hello".into()));
        let distribute = InscriptionDistribute::new(7);
        let occupy = InscriptionOccupy::new(9);

        let inscribe_json: serde_json::Value =
            serde_json::from_slice(&encode_inscription_inscribe(&inscribe).unwrap()).unwrap();
        let distribute_json: serde_json::Value =
            serde_json::from_slice(&encode_inscription_distribute(&distribute).unwrap()).unwrap();
        let occupy_json: serde_json::Value =
            serde_json::from_slice(&encode_inscription_occupy(&occupy).unwrap()).unwrap();

        assert_eq!(inscribe_json["operation"], INSCRIPTION_OPERATION_INSCRIBE);
        assert_eq!(inscribe_json["recipient"], "MIX-address");
        assert_eq!(
            distribute_json["distribute"],
            INSCRIPTION_OPERATION_DISTRIBUTE
        );
        assert_eq!(distribute_json["sequence"], 7);
        assert_eq!(occupy_json["operation"], INSCRIPTION_OPERATION_OCCUPY);
        assert_eq!(occupy_json["sequence"], 9);
    }
}
