use serde::{Deserialize, Serialize};

use crate::{
    error::Error,
    request::{ApiResponse, request},
};

pub const LIGHTNING_CHAIN_ID: &str = "59c09123-95cc-3ffd-a659-0f9169074cee";
pub const BITCOIN_CHAIN_ID: &str = "c6d0c728-2624-429b-8e0d-d9d19b6592fa";
pub const BITCOIN_CASH_CHAIN_ID: &str = "fd11b6e3-0b87-41f1-a41f-f0e9b49e5bf0";
pub const BITCOIN_SV_CHAIN_ID: &str = "574388fd-b93f-4034-a682-01c2bc095d17";
pub const LITECOIN_CHAIN_ID: &str = "76c802a2-7c88-447f-a93e-c29c9e5dd9c8";
pub const ETHEREUM_CHAIN_ID: &str = "43d61dcd-e413-450d-80b8-101d5e903357";
pub const ETHEREUM_CLASSIC_CHAIN_ID: &str = "2204c1ee-0ea2-4add-bb9a-b3719cfff93a";
pub const BSC_CHAIN_ID: &str = "1949e683-6a08-49e2-b087-d6b72398588f";
pub const POLYGON_CHAIN_ID: &str = "b7938396-3f94-4e0a-9179-d3440718156f";
pub const BASE_CHAIN_ID: &str = "3fb612c5-6844-3979-ae4a-5a84e79da870";
pub const OPTIMISM_CHAIN_ID: &str = "60360611-370c-3b69-9826-b13db93f6aba";
pub const ARBITRUM_CHAIN_ID: &str = "8c590110-1abc-3697-84f2-05214e6516aa";
pub const MVM_CHAIN_ID: &str = "a0ffd769-5850-4b48-9651-d2ae44a3e64d";
pub const DECRED_CHAIN_ID: &str = "8f5caf2a-283d-4c85-832a-91e83bbf290b";
pub const RIPPLE_CHAIN_ID: &str = "23dfb5a5-5d7b-48b6-905f-3970e3176e27";
pub const SIACOIN_CHAIN_ID: &str = "990c4c29-57e9-48f6-9819-7d986ea44985";
pub const EOS_CHAIN_ID: &str = "6cfe566e-4aad-470b-8c9a-2fd35b49c68d";
pub const DOGECOIN_CHAIN_ID: &str = "6770a1e5-6086-44d5-b60f-545f9d9e8ffd";
pub const DASH_CHAIN_ID: &str = "6472e7e3-75fd-48b6-b1dc-28d294ee1476";
pub const ZCASH_CHAIN_ID: &str = "c996abc9-d94e-4494-b1cf-2a3fd3ac5714";
pub const NEM_CHAIN_ID: &str = "27921032-f73e-434e-955f-43d55672ee31";
pub const ARWEAVE_CHAIN_ID: &str = "882eb041-64ea-465f-a4da-817bd3020f52";
pub const HORIZEN_CHAIN_ID: &str = "a2c5d22b-62a2-4c13-b3f0-013290dbac60";
pub const TRON_CHAIN_ID: &str = "25dabac5-056a-48ff-b9f9-f67395dc407c";
pub const STELLAR_CHAIN_ID: &str = "56e63c06-b506-4ec5-885a-4a5ac17b83c1";
pub const MASS_GRID_CHAIN_ID: &str = "b207bce9-c248-4b8e-b6e3-e357146f3f4c";
pub const BYTOM_CHAIN_ID: &str = "443e1ef5-bc9b-47d3-be77-07f328876c50";
pub const BYTOM_POS_CHAIN_ID: &str = "71a0e8b5-a289-4845-b661-2b70ff9968aa";
pub const COSMOS_CHAIN_ID: &str = "7397e9f1-4e42-4dc8-8a3b-171daaadd436";
pub const AKASH_CHAIN_ID: &str = "9c612618-ca59-4583-af34-be9482f5002d";
pub const BINANCE_CHAIN_ID: &str = "17f78d7c-ed96-40ff-980c-5dc62fecbc85";
pub const MONERO_CHAIN_ID: &str = "05c5ac01-31f9-4a69-aa8a-ab796de1d041";
pub const STARCOIN_CHAIN_ID: &str = "c99a3779-93df-404d-945d-eddc440aa0b2";
pub const BITSHARES_CHAIN_ID: &str = "05891083-63d2-4f3d-bfbe-d14d7fb9b25a";
pub const RAVENCOIN_CHAIN_ID: &str = "6877d485-6b64-4225-8d7e-7333393cb243";
pub const GRIN_CHAIN_ID: &str = "1351e6bd-66cf-40c1-8105-8a8fe518a222";
pub const VCASH_CHAIN_ID: &str = "c3b9153a-7fab-4138-a3a4-99849cadc073";
pub const HANDSHAKE_CHAIN_ID: &str = "13036886-6b83-4ced-8d44-9f69151587bf";
pub const NERVOS_CHAIN_ID: &str = "d243386e-6d84-42e6-be03-175be17bf275";
pub const TEZOS_CHAIN_ID: &str = "5649ca42-eb5f-4c0e-ae28-d9a4e77eded3";
pub const NAMECOIN_CHAIN_ID: &str = "f8b77dc0-46fd-4ea1-9821-587342475869";
pub const SOLANA_CHAIN_ID: &str = "64692c23-8971-4cf4-84a7-4dd1271dd887";
pub const NEAR_CHAIN_ID: &str = "d6ac94f7-c932-4e11-97dd-617867f0669e";
pub const FILECOIN_CHAIN_ID: &str = "08285081-e1d8-4be6-9edc-e203afa932da";
pub const MOBILECOIN_CHAIN_ID: &str = "eea900a8-b327-488c-8d8d-1428702fe240";
pub const POLKADOT_CHAIN_ID: &str = "54c61a72-b982-4034-a556-0d99e3c21e39";
pub const KUSAMA_CHAIN_ID: &str = "9d29e4f6-d67c-4c4b-9525-604b04afbe9f";
pub const ALGORAND_CHAIN_ID: &str = "706b6f84-3333-4e55-8e89-275e71ce9803";
pub const AVALANCHE_X_CHAIN_ID: &str = "cbc77539-0a20-4666-8c8a-4ded62b36f0a";
pub const AVALANCHE_C_CHAIN_ID: &str = "1f67ac58-87ba-3571-9781-e9413c046f34";
pub const MARS_CHAIN_CHAIN_ID: &str = "163a2142-398d-3483-aee3-d47db8da4d10";
pub const XDC_CHAIN_ID: &str = "b12bb04a-1cea-401c-a086-0be61f544889";
pub const APTOS_CHAIN_ID: &str = "d2c1c7e1-a1a9-4f88-b282-d93b0a08b42b";
pub const SUI_CHAIN_ID: &str = "2bd97283-2582-33a8-bcba-f4b8ed189572";
pub const TON_CHAIN_ID: &str = "ef660437-d915-4e27-ad3f-632bfb6ba0ee";

pub const XIN_ASSET_ID: &str = "c94ac88f-4671-3976-b60a-09064f1811e8";
pub const VAULTA_ASSET_ID: &str = "ac2b79f3-ec9c-3d87-b4ca-3e825228dda5";

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq)]
pub struct NetworkChain {
    #[serde(default, rename = "type")]
    pub type_name: Option<String>,
    pub chain_id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub managed_block_height: Option<i64>,
    #[serde(default)]
    pub deposit_block_height: Option<i64>,
    #[serde(default)]
    pub external_block_height: Option<i64>,
    #[serde(default)]
    pub threshold: Option<i64>,
    #[serde(default)]
    pub withdrawal_timestamp: Option<String>,
    #[serde(default)]
    pub withdrawal_pending_count: Option<i64>,
    #[serde(default)]
    pub withdrawal_fee: Option<String>,
    #[serde(default)]
    pub is_synchronized: Option<bool>,
}

pub async fn read_network_chain(chain_id: &str) -> Result<NetworkChain, Error> {
    let path = format!("/network/chains/{chain_id}");
    let body = request("GET", &path, &[], "").await?;
    parse_data(&body, "network chain")
}

pub async fn read_network_chains() -> Result<Vec<NetworkChain>, Error> {
    let path = "/network/chains";
    let body = request("GET", path, &[], "").await?;
    parse_data(&body, "network chains")
}

pub fn get_chain_name(chain_id: &str) -> &'static str {
    match chain_id {
        EOS_CHAIN_ID => "EOS",
        RIPPLE_CHAIN_ID => "Ripple",
        SIACOIN_CHAIN_ID => "Siacoin",
        ETHEREUM_CHAIN_ID => "Ethereum",
        ETHEREUM_CLASSIC_CHAIN_ID => "Ethereum Classic",
        BSC_CHAIN_ID => "BNB Smart Chain",
        POLYGON_CHAIN_ID => "Polygon",
        BASE_CHAIN_ID => "Base",
        OPTIMISM_CHAIN_ID => "OP Mainnet",
        ARBITRUM_CHAIN_ID => "Arbitrum One",
        MVM_CHAIN_ID => "Mixin Virtual Machine",
        BITCOIN_CHAIN_ID => "Bitcoin",
        HANDSHAKE_CHAIN_ID => "Handshake",
        BITCOIN_CASH_CHAIN_ID => "Bitcoin Cash",
        BITCOIN_SV_CHAIN_ID => "Bitcoin SV",
        LITECOIN_CHAIN_ID => "Litecoin",
        DECRED_CHAIN_ID => "Decred",
        DOGECOIN_CHAIN_ID => "Dogecoin",
        DASH_CHAIN_ID => "Dash",
        ZCASH_CHAIN_ID => "Zcash",
        AVALANCHE_X_CHAIN_ID => "Avalanche X-Chain",
        AVALANCHE_C_CHAIN_ID => "Avalanche C-Chain",
        MARS_CHAIN_CHAIN_ID => "MarsChain",
        MONERO_CHAIN_ID => "Monero",
        NEM_CHAIN_ID => "NEM",
        HORIZEN_CHAIN_ID => "Horizen",
        MASS_GRID_CHAIN_ID => "MassGrid",
        BYTOM_CHAIN_ID => "Bytom",
        BYTOM_POS_CHAIN_ID => "Bytom",
        TRON_CHAIN_ID => "TRON",
        TON_CHAIN_ID => "TON",
        STELLAR_CHAIN_ID => "Stellar",
        COSMOS_CHAIN_ID => "Cosmos",
        STARCOIN_CHAIN_ID => "Starcoin",
        AKASH_CHAIN_ID => "Akash",
        BINANCE_CHAIN_ID => "BNB Beacon Chain",
        BITSHARES_CHAIN_ID => "Bitshares",
        TEZOS_CHAIN_ID => "Tezos",
        RAVENCOIN_CHAIN_ID => "Ravencoin",
        NAMECOIN_CHAIN_ID => "Namecoin",
        NERVOS_CHAIN_ID => "Nervos",
        GRIN_CHAIN_ID => "Grin",
        VCASH_CHAIN_ID => "VCash",
        FILECOIN_CHAIN_ID => "Filecoin",
        POLKADOT_CHAIN_ID => "Polkadot",
        KUSAMA_CHAIN_ID => "Kusama",
        ARWEAVE_CHAIN_ID => "Arweave",
        MOBILECOIN_CHAIN_ID => "MobileCoin",
        SOLANA_CHAIN_ID => "Solana",
        NEAR_CHAIN_ID => "NEAR",
        ALGORAND_CHAIN_ID => "Algorand",
        XDC_CHAIN_ID => "XDC Network",
        APTOS_CHAIN_ID => "Aptos",
        SUI_CHAIN_ID => "Sui",
        _ => "Not Supported Chain",
    }
}

pub fn is_chain_id(chain_id: &str) -> bool {
    full_chain_ids().contains(&chain_id)
}

pub fn full_chain_ids() -> &'static [&'static str] {
    &[
        BITCOIN_CHAIN_ID,
        BITCOIN_CASH_CHAIN_ID,
        BITCOIN_SV_CHAIN_ID,
        LITECOIN_CHAIN_ID,
        ETHEREUM_CHAIN_ID,
        ETHEREUM_CLASSIC_CHAIN_ID,
        BSC_CHAIN_ID,
        POLYGON_CHAIN_ID,
        BASE_CHAIN_ID,
        OPTIMISM_CHAIN_ID,
        ARBITRUM_CHAIN_ID,
        MVM_CHAIN_ID,
        DECRED_CHAIN_ID,
        RIPPLE_CHAIN_ID,
        SIACOIN_CHAIN_ID,
        EOS_CHAIN_ID,
        DOGECOIN_CHAIN_ID,
        DASH_CHAIN_ID,
        ZCASH_CHAIN_ID,
        NEM_CHAIN_ID,
        ARWEAVE_CHAIN_ID,
        HORIZEN_CHAIN_ID,
        TRON_CHAIN_ID,
        STELLAR_CHAIN_ID,
        MASS_GRID_CHAIN_ID,
        BYTOM_CHAIN_ID,
        BYTOM_POS_CHAIN_ID,
        COSMOS_CHAIN_ID,
        AKASH_CHAIN_ID,
        BINANCE_CHAIN_ID,
        MONERO_CHAIN_ID,
        STARCOIN_CHAIN_ID,
        BITSHARES_CHAIN_ID,
        RAVENCOIN_CHAIN_ID,
        GRIN_CHAIN_ID,
        VCASH_CHAIN_ID,
        HANDSHAKE_CHAIN_ID,
        NERVOS_CHAIN_ID,
        TEZOS_CHAIN_ID,
        NAMECOIN_CHAIN_ID,
        SOLANA_CHAIN_ID,
        NEAR_CHAIN_ID,
        FILECOIN_CHAIN_ID,
        MOBILECOIN_CHAIN_ID,
        POLKADOT_CHAIN_ID,
        KUSAMA_CHAIN_ID,
        ALGORAND_CHAIN_ID,
        AVALANCHE_X_CHAIN_ID,
        AVALANCHE_C_CHAIN_ID,
        MARS_CHAIN_CHAIN_ID,
        XDC_CHAIN_ID,
        APTOS_CHAIN_ID,
        TON_CHAIN_ID,
    ]
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
    fn test_chain_name_and_validation() {
        assert_eq!(get_chain_name(BITCOIN_CHAIN_ID), "Bitcoin");
        assert_eq!(get_chain_name("missing"), "Not Supported Chain");
        assert!(is_chain_id(ETHEREUM_CHAIN_ID));
        assert!(!is_chain_id("missing"));
    }

    #[test]
    fn test_network_chain_deserialize() {
        let raw = r#"{
            "type": "chain",
            "chain_id": "c6d0c728-2624-429b-8e0d-d9d19b6592fa",
            "name": "Bitcoin",
            "symbol": "BTC",
            "threshold": 12,
            "withdrawal_fee": "0.0001",
            "is_synchronized": true
        }"#;
        let chain: NetworkChain = serde_json::from_str(raw).expect("chain");
        assert_eq!(chain.chain_id, BITCOIN_CHAIN_ID);
        assert_eq!(chain.name.as_deref(), Some("Bitcoin"));
        assert_eq!(chain.threshold, Some(12));
        assert_eq!(chain.is_synchronized, Some(true));
    }
}
