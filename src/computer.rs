use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use reqwest::{
    StatusCode,
    header::{CONTENT_LENGTH, CONTENT_TYPE},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;

use crate::{
    chain::SOLANA_CHAIN_ID,
    error::Error,
    mix_address::{MixAddress, new_uuid_mix_address},
    request::{ApiError, HTTP_CLIENT},
    safe::SafeUser,
    safe_transaction::{SafeTransactionRecipient, send_safe_transaction},
    transaction::TransactionView,
    utils::unique_object_id,
};

pub const COMPUTER_URI: &str = "https://computer.mixin.one";
pub const MAX_SOLANA_TX_SIZE: usize = 1232;

pub const OPERATION_TYPE_ADD_USER: u8 = 1;
pub const OPERATION_TYPE_SYSTEM_CALL: u8 = 2;
pub const OPERATION_TYPE_USER_DEPOSIT: u8 = 3;

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq, Eq)]
pub struct ComputerInfo {
    pub observer: String,
    pub payer: String,
    pub height: i64,
    pub members: ComputerMembers,
    pub params: ComputerParams,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq, Eq)]
pub struct ComputerMembers {
    pub app_id: String,
    pub members: Vec<String>,
    pub threshold: u8,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq, Eq)]
pub struct ComputerParams {
    pub operation: ComputerOperationParams,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq, Eq)]
pub struct ComputerOperationParams {
    pub asset: String,
    pub price: String,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq, Eq)]
pub struct ComputerUser {
    pub id: String,
    pub chain_address: String,
    pub mix_address: String,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq, Eq)]
pub struct ComputerDeployedAsset {
    pub asset_id: String,
    pub chain_id: String,
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: i64,
    pub price_usd: String,
    #[serde(rename = "uri")]
    pub icon_url: String,
}

impl ComputerDeployedAsset {
    pub fn solana_asset_id(&self) -> String {
        unique_object_id([SOLANA_CHAIN_ID, self.address.as_str()])
    }
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq, Eq)]
pub struct ComputerSystemCall {
    #[serde(default)]
    pub id: String,
    #[serde(default, rename = "type")]
    pub type_name: String,
    #[serde(default)]
    pub user_id: String,
    #[serde(default)]
    pub nonce_account: String,
    #[serde(default)]
    pub raw: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub hash: String,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq, Eq)]
pub struct ComputerSystemCallResponse {
    #[serde(flatten)]
    pub call: ComputerSystemCall,
    #[serde(default)]
    pub reason: String,
    #[serde(default, rename = "subs")]
    pub sub_calls: Vec<ComputerSystemCall>,
    #[serde(default)]
    pub refund_traces: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq, Eq)]
pub struct ComputerNonceAccount {
    pub mix: String,
    pub nonce_address: String,
    pub nonce_hash: String,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq, Eq)]
pub struct ComputerFee {
    pub fee_id: String,
    pub xin_amount: String,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq, Eq)]
pub struct ComputerRegisterPreview {
    pub mix_address: String,
    pub trace_id: String,
    pub mtg_extra: String,
    pub operation_asset: String,
    pub operation_price: String,
    pub mtg_app_id: String,
    pub mtg_members: Vec<String>,
    pub mtg_threshold: u8,
}

#[derive(Debug, Serialize)]
struct LockNonceAccountRequest<'a> {
    mix: &'a str,
}

#[derive(Debug, Serialize)]
struct FeeRequest<'a> {
    sol_amount: &'a str,
}

pub async fn get_computer_info() -> Result<ComputerInfo, Error> {
    let (status, body) = computer_request("GET", "/", &[]).await?;
    parse_computer_response(status, &body, "computer info")
}

pub async fn get_computer_user(address: &str) -> Result<Option<ComputerUser>, Error> {
    let path = format!("/users/{address}");
    let (status, body) = computer_request("GET", &path, &[]).await?;
    match parse_computer_response(status, &body, "computer user") {
        Ok(user) => Ok(Some(user)),
        Err(Error::Api(error)) if is_not_found(&error) => Ok(None),
        Err(error) => Err(error),
    }
}

pub async fn get_computer_deployed_assets() -> Result<Vec<ComputerDeployedAsset>, Error> {
    let (status, body) = computer_request("GET", "/deployed_assets", &[]).await?;
    parse_computer_response(status, &body, "computer deployed assets")
}

pub async fn get_computer_system_call(
    id: &str,
) -> Result<Option<ComputerSystemCallResponse>, Error> {
    let path = format!("/system_calls/{id}");
    let (status, body) = computer_request("GET", &path, &[]).await?;
    match parse_computer_response(status, &body, "computer system call") {
        Ok(call) => Ok(Some(call)),
        Err(Error::Api(error)) if is_not_found(&error) => Ok(None),
        Err(error) => Err(error),
    }
}

pub async fn computer_deploy_external_assets(asset_ids: &[String]) -> Result<(), Error> {
    for asset_id in asset_ids {
        if asset_id == SOLANA_CHAIN_ID {
            return Err(Error::Input(format!(
                "cannot deploy asset from Solana: {asset_id}"
            )));
        }
    }
    let body = serde_json::to_vec(asset_ids)?;
    let (status, body) = computer_request("POST", "/deployed_assets", &body).await?;
    parse_computer_empty_response(status, &body)
}

pub async fn lock_computer_nonce_account(mix: &str) -> Result<ComputerNonceAccount, Error> {
    let body = serde_json::to_vec(&LockNonceAccountRequest { mix })?;
    let (status, body) = computer_request("POST", "/nonce_accounts", &body).await?;
    parse_computer_response(status, &body, "computer nonce account")
}

pub async fn get_fee_on_xin_based_on_sol(sol_amount: &str) -> Result<ComputerFee, Error> {
    let body = serde_json::to_vec(&FeeRequest { sol_amount })?;
    let (status, body) = computer_request("POST", "/fee", &body).await?;
    parse_computer_response(status, &body, "computer fee")
}

pub fn computer_mix_address_for_user(user_id: &str) -> Result<String, Error> {
    Ok(new_uuid_mix_address(vec![user_id.to_string()], 1)?.to_string())
}

pub fn computer_register_trace(mix_address: &str) -> String {
    unique_object_id([mix_address, "computer_register"])
}

pub fn build_computer_register_preview(
    info: &ComputerInfo,
    safe_user: &SafeUser,
) -> Result<ComputerRegisterPreview, Error> {
    let mix_address = computer_mix_address_for_user(&safe_user.user_id)?;
    let memo = encode_operation_memo(OPERATION_TYPE_ADD_USER, mix_address.as_bytes());
    let mtg_extra = encode_mtg_extra(&info.members.app_id, &memo)?;
    Ok(ComputerRegisterPreview {
        trace_id: computer_register_trace(&mix_address),
        mix_address,
        mtg_extra,
        operation_asset: info.params.operation.asset.clone(),
        operation_price: info.params.operation.price.clone(),
        mtg_app_id: info.members.app_id.clone(),
        mtg_members: info.members.members.clone(),
        mtg_threshold: info.members.threshold,
    })
}

pub async fn preview_register_computer(
    safe_user: &SafeUser,
) -> Result<ComputerRegisterPreview, Error> {
    let info = get_computer_info().await?;
    build_computer_register_preview(&info, safe_user)
}

pub async fn register_computer(safe_user: &SafeUser) -> Result<TransactionView, Error> {
    let info = get_computer_info().await?;
    let preview = build_computer_register_preview(&info, safe_user)?;
    let mtg_mix_address =
        MixAddress::new_uuid(info.members.members.clone(), info.members.threshold)?;
    let recipient =
        SafeTransactionRecipient::mix_address(mtg_mix_address, info.params.operation.price);
    send_safe_transaction(
        &info.params.operation.asset,
        &[recipient],
        &preview.trace_id,
        preview.mtg_extra.into_bytes(),
        Vec::new(),
        safe_user,
    )
    .await
}

pub fn check_system_call_size(tx: &[u8]) -> bool {
    tx.len() <= MAX_SOLANA_TX_SIZE
}

pub fn computer_user_id_to_bytes(id: &str) -> Result<[u8; 8], Error> {
    let value = id
        .parse::<u64>()
        .map_err(|err| Error::Input(format!("invalid computer user id {id}: {err}")))?;
    Ok(value.to_be_bytes())
}

pub fn build_system_call_extra(
    uid: &str,
    call_id: &str,
    skip_process: bool,
    fee_id: Option<&str>,
) -> Result<Vec<u8>, Error> {
    let mut extra = Vec::with_capacity(8 + 16 + 1 + fee_id.map_or(0, |_| 16));
    extra.extend_from_slice(&computer_user_id_to_bytes(uid)?);
    extra.extend_from_slice(
        Uuid::parse_str(call_id)
            .map_err(|err| Error::Input(format!("invalid call id {call_id}: {err}")))?
            .as_bytes(),
    );
    extra.push(u8::from(skip_process));
    if let Some(fee_id) = fee_id {
        extra.extend_from_slice(
            Uuid::parse_str(fee_id)
                .map_err(|err| Error::Input(format!("invalid fee id {fee_id}: {err}")))?
                .as_bytes(),
        );
    }
    Ok(extra)
}

pub fn encode_operation_memo(operation: u8, extra: &[u8]) -> Vec<u8> {
    let mut memo = Vec::with_capacity(1 + extra.len());
    memo.push(operation);
    memo.extend_from_slice(extra);
    memo
}

pub fn encode_mtg_extra(app_id: &str, extra: &[u8]) -> Result<String, Error> {
    let mut data = Vec::with_capacity(16 + extra.len());
    data.extend_from_slice(
        Uuid::parse_str(app_id)
            .map_err(|err| Error::Input(format!("invalid computer app id {app_id}: {err}")))?
            .as_bytes(),
    );
    data.extend_from_slice(extra);
    Ok(URL_SAFE_NO_PAD.encode(data))
}

pub fn decode_computer_extra_base64(extra: &str) -> Result<(String, Vec<u8>), Error> {
    let data = URL_SAFE_NO_PAD
        .decode(extra)
        .map_err(|err| Error::Input(format!("invalid computer extra base64: {err}")))?;
    if data.len() < 16 {
        return Err(Error::Input(format!(
            "invalid computer extra length: {}",
            data.len()
        )));
    }
    let app_id = Uuid::from_slice(&data[..16])
        .map_err(|err| Error::Input(format!("invalid computer app id bytes: {err}")))?
        .to_string();
    Ok((app_id, data[16..].to_vec()))
}

async fn computer_request(
    method: &str,
    path: &str,
    body: &[u8],
) -> Result<(StatusCode, Vec<u8>), Error> {
    let uri = format!("{COMPUTER_URI}{path}");
    let method = reqwest::Method::from_bytes(method.as_bytes())?;
    let mut request_builder = HTTP_CLIENT
        .request(method.clone(), &uri)
        .header(CONTENT_TYPE, "application/json");
    if method != reqwest::Method::GET {
        request_builder = request_builder.header(CONTENT_LENGTH, body.len());
    }
    let response = request_builder.body(body.to_vec()).send().await?;
    let status = response.status();
    let body = response.bytes().await?.to_vec();
    if status.is_server_error() {
        return Err(Error::Api(computer_http_error(status, &body)));
    }
    Ok((status, body))
}

fn parse_computer_response<T>(status: StatusCode, body: &[u8], label: &str) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    let value = parse_computer_json(status, body, label)?;
    if let Some(error) = computer_error_from_value(status, &value) {
        return Err(Error::Api(error));
    }
    Ok(serde_json::from_value(value)?)
}

fn parse_computer_empty_response(status: StatusCode, body: &[u8]) -> Result<(), Error> {
    if body.iter().all(|byte| byte.is_ascii_whitespace()) {
        return if status.is_success() {
            Ok(())
        } else {
            Err(Error::Api(computer_http_error(status, body)))
        };
    }
    let value = parse_computer_json(status, body, "computer response")?;
    if let Some(error) = computer_error_from_value(status, &value) {
        return Err(Error::Api(error));
    }
    if status.is_success() {
        Ok(())
    } else {
        Err(Error::Api(computer_http_error(status, body)))
    }
}

fn parse_computer_json(
    status: StatusCode,
    body: &[u8],
    label: &str,
) -> Result<serde_json::Value, Error> {
    if body.iter().all(|byte| byte.is_ascii_whitespace()) {
        if status.is_success() {
            return Err(Error::DataNotFound(label.to_string()));
        }
        return Err(Error::Api(computer_http_error(status, body)));
    }
    match serde_json::from_slice(body) {
        Ok(value) => Ok(value),
        Err(err) if status.is_success() => Err(Error::Json(err)),
        Err(_) => Err(Error::Api(computer_http_error(status, body))),
    }
}

fn computer_error_from_value(status: StatusCode, value: &serde_json::Value) -> Option<ApiError> {
    if let Some(error_value) = value.get("error").filter(|value| !value.is_null()) {
        if let Ok(mut error) = serde_json::from_value::<ApiError>(error_value.clone())
            && has_api_error(&error)
        {
            if error.status == 0 {
                error.status = status.as_u16() as i32;
            }
            return Some(error);
        }
        if let Some(description) = error_value.as_str() {
            return Some(ApiError {
                status: status.as_u16() as i32,
                code: status.as_u16() as i32,
                description: description.to_string(),
            });
        }
    }

    let code = value
        .get("code")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or_default() as i32;
    let status_field = value
        .get("status")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or_default() as i32;
    if code > 0 || status_field >= 400 || !status.is_success() {
        let description = value
            .get("description")
            .or_else(|| value.get("message"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| status.canonical_reason().unwrap_or("Computer API error"))
            .to_string();
        return Some(ApiError {
            status: if status_field > 0 {
                status_field
            } else {
                status.as_u16() as i32
            },
            code: if code > 0 {
                code
            } else {
                status.as_u16() as i32
            },
            description,
        });
    }
    None
}

fn computer_http_error(status: StatusCode, body: &[u8]) -> ApiError {
    if let Ok(value) = serde_json::from_slice::<serde_json::Value>(body)
        && let Some(error) = computer_error_from_value(status, &value)
    {
        return error;
    }
    let description = String::from_utf8_lossy(body).trim().to_string();
    ApiError {
        status: status.as_u16() as i32,
        code: status.as_u16() as i32,
        description: if description.is_empty() {
            status
                .canonical_reason()
                .unwrap_or("Computer API error")
                .to_string()
        } else {
            description
        },
    }
}

fn has_api_error(error: &ApiError) -> bool {
    error.code > 0 || error.status >= 400 || !error.description.is_empty()
}

fn is_not_found(error: &ApiError) -> bool {
    error.code == 404 || error.status == 404
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::safe::SafeUser;

    #[test]
    fn test_computer_user_id_to_bytes() {
        assert_eq!(
            computer_user_id_to_bytes("1").unwrap(),
            [0, 0, 0, 0, 0, 0, 0, 1]
        );
        assert_eq!(
            computer_user_id_to_bytes("18446744073709551615").unwrap(),
            [255; 8]
        );
        assert!(computer_user_id_to_bytes("-1").is_err());
    }

    #[test]
    fn test_build_system_call_extra() {
        let extra = build_system_call_extra(
            "1",
            "00000000-0000-0000-0000-000000000002",
            true,
            Some("00000000-0000-0000-0000-000000000003"),
        )
        .unwrap();
        assert_eq!(extra.len(), 41);
        assert_eq!(&extra[..8], &[0, 0, 0, 0, 0, 0, 0, 1]);
        assert_eq!(extra[24], 1);
    }

    #[test]
    fn test_encode_decode_mtg_extra() {
        let app_id = "00000000-0000-0000-0000-000000000001";
        let memo = encode_operation_memo(OPERATION_TYPE_SYSTEM_CALL, &[1, 2, 3]);
        let encoded = encode_mtg_extra(app_id, &memo).unwrap();
        let (decoded_app_id, decoded_memo) = decode_computer_extra_base64(&encoded).unwrap();
        assert_eq!(decoded_app_id, app_id);
        assert_eq!(decoded_memo, memo);
    }

    #[test]
    fn test_check_system_call_size() {
        assert!(check_system_call_size(&vec![0; MAX_SOLANA_TX_SIZE]));
        assert!(!check_system_call_size(&vec![0; MAX_SOLANA_TX_SIZE + 1]));
    }

    #[test]
    fn test_deployed_asset_solana_asset_id() {
        let asset = ComputerDeployedAsset {
            address: "So11111111111111111111111111111111111111112".to_string(),
            ..Default::default()
        };
        assert_eq!(
            asset.solana_asset_id(),
            unique_object_id([
                SOLANA_CHAIN_ID,
                "So11111111111111111111111111111111111111112"
            ])
        );
    }

    #[test]
    fn test_register_preview_matches_go_layout() {
        let safe_user = SafeUser {
            user_id: "67a87828-18f5-46a1-b6cc-c72a97a77c43".to_string(),
            session_id: "session-id".to_string(),
            session_private_key: "session-private-key".to_string(),
            server_public_key: "server-public-key".to_string(),
            spend_private_key: "spend-private-key".to_string(),
            is_spend_private_sum: false,
        };
        let info = ComputerInfo {
            members: ComputerMembers {
                app_id: "00000000-0000-0000-0000-000000000001".to_string(),
                members: vec!["00000000-0000-0000-0000-000000000002".to_string()],
                threshold: 1,
            },
            params: ComputerParams {
                operation: ComputerOperationParams {
                    asset: "asset".to_string(),
                    price: "0.01".to_string(),
                },
            },
            ..Default::default()
        };

        let preview = build_computer_register_preview(&info, &safe_user).unwrap();
        let (app_id, memo) = decode_computer_extra_base64(&preview.mtg_extra).unwrap();
        assert_eq!(app_id, info.members.app_id);
        assert_eq!(memo[0], OPERATION_TYPE_ADD_USER);
        assert_eq!(
            String::from_utf8(memo[1..].to_vec()).unwrap(),
            preview.mix_address
        );
        assert_eq!(
            preview.trace_id,
            unique_object_id([preview.mix_address.as_str(), "computer_register"])
        );
    }
}
