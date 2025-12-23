use serde::{Deserialize, Serialize};
use url::form_urlencoded;

use crate::{
    auth::sign_authentication_token,
    error::Error,
    pin::encrypt_ed25519_pin,
    request::{ApiResponse, request},
    safe::SafeUser,
    tip::{TIP_ADDRESS_REMOVE, sign_tip_body, tip_body, tip_body_for_address_add},
};

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Address {
    pub address_id: String,
    #[serde(default)]
    pub asset_id: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub destination: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub fee: Option<String>,
    #[serde(default)]
    pub dust: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct SimpleAddress {
    pub destination: Option<String>,
    pub tag: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AddressInput<'a> {
    pub asset_id: &'a str,
    pub label: &'a str,
    pub destination: &'a str,
    pub tag: &'a str,
}

pub async fn create_address(
    input: &AddressInput<'_>,
    safe_user: &SafeUser,
) -> Result<Address, Error> {
    let tip_body =
        tip_body_for_address_add(input.asset_id, input.destination, input.tag, input.label);
    let pin = sign_tip_body(
        &tip_body,
        &safe_user.spend_private_key,
        safe_user.is_spend_private_sum,
    )?;
    let pin_base64 = encrypt_ed25519_pin(&pin, now_nanos()?, safe_user)?;

    let data = serde_json::json!({
        "asset_id": input.asset_id,
        "label": input.label,
        "destination": input.destination,
        "tag": input.tag,
        "pin_base64": pin_base64,
    });
    let data_str = data.to_string();
    let path = "/addresses";
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<Address> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain address data".to_string()))
}

pub async fn read_address(address_id: &str, safe_user: &SafeUser) -> Result<Address, Error> {
    let path = format!("/addresses/{address_id}");
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    let parsed: ApiResponse<Address> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain address data".to_string()))
}

pub async fn delete_address(address_id: &str, safe_user: &SafeUser) -> Result<(), Error> {
    let tip = tip_body(&(TIP_ADDRESS_REMOVE.to_string() + address_id));
    let pin = sign_tip_body(
        &tip,
        &safe_user.spend_private_key,
        safe_user.is_spend_private_sum,
    )?;
    let pin_base64 = encrypt_ed25519_pin(&pin, now_nanos()?, safe_user)?;

    let data = serde_json::json!({
        "pin_base64": pin_base64,
    });
    let data_str = data.to_string();

    let path = format!("/addresses/{address_id}/delete");
    let token = sign_authentication_token("POST", &path, &data_str, safe_user)?;
    let body = request("POST", &path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<serde_json::Value> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    Ok(())
}

pub async fn list_addresses_by_asset(
    asset_id: &str,
    safe_user: &SafeUser,
) -> Result<Vec<Address>, Error> {
    let path = format!("/assets/{asset_id}/addresses");
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    let parsed: ApiResponse<Vec<Address>> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain address data".to_string()))
}

pub async fn check_address(
    asset: &str,
    destination: &str,
    tag: Option<&str>,
) -> Result<SimpleAddress, Error> {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    serializer.append_pair("asset", asset);
    serializer.append_pair("destination", destination);
    if let Some(tag) = tag && !tag.is_empty() {
        serializer.append_pair("tag", tag);
    }
    let query = serializer.finish();
    let path = format!("/external/addresses/check?{query}");
    let body = request("GET", &path, &[], "").await?;

    let parsed: ApiResponse<SimpleAddress> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain address data".to_string()))
}

fn now_nanos() -> Result<u64, Error> {
    use std::time::{SystemTime, UNIX_EPOCH};
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Error::Server(e.to_string()))?
        .as_nanos() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_input_serialization() {
        let input = AddressInput {
            asset_id: "asset-id",
            label: "label",
            destination: "dest",
            tag: "tag",
        };
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&input).unwrap()).unwrap();
        assert_eq!(value["asset_id"], "asset-id");
        assert_eq!(value["label"], "label");
        assert_eq!(value["destination"], "dest");
        assert_eq!(value["tag"], "tag");
    }
}
