use serde::Serialize;
use url::form_urlencoded;

use crate::{
    auth::sign_authentication_token,
    error::Error,
    models::Output,
    request::{ApiResponse, request},
    safe::SafeUser,
};

pub const OUTPUT_STATE_UNSPENT: &str = "unspent";
pub const OUTPUT_STATE_SIGNED: &str = "signed";
pub const OUTPUT_STATE_SPENT: &str = "spent";

#[derive(Debug, Serialize)]
pub struct OutputFetchRequest<'a> {
    pub user_id: &'a str,
    pub ids: &'a [String],
}

pub async fn list_outputs(
    members_hash: &str,
    threshold: u8,
    asset_id: Option<&str>,
    state: Option<&str>,
    offset: Option<i64>,
    limit: Option<i64>,
    safe_user: &SafeUser,
) -> Result<Vec<Output>, Error> {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    serializer.append_pair("members", members_hash);
    serializer.append_pair("threshold", &threshold.to_string());
    serializer.append_pair("limit", &limit.unwrap_or(500).to_string());
    if let Some(offset) = offset
        && offset > 0
    {
        serializer.append_pair("offset", &offset.to_string());
    }
    if let Some(asset_id) = asset_id
        && !asset_id.is_empty()
    {
        serializer.append_pair("asset", asset_id);
    }
    if let Some(state) = state
        && !state.is_empty()
    {
        serializer.append_pair("state", state);
    }

    let query = serializer.finish();
    let path = format!("/safe/outputs?{query}");
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    let parsed: ApiResponse<Vec<Output>> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain output data".to_string()))
}

pub async fn list_unspent_outputs(
    members_hash: &str,
    threshold: u8,
    asset_id: Option<&str>,
    safe_user: &SafeUser,
) -> Result<Vec<Output>, Error> {
    list_outputs(
        members_hash,
        threshold,
        asset_id,
        Some(OUTPUT_STATE_UNSPENT),
        Some(0),
        Some(500),
        safe_user,
    )
    .await
}

pub async fn get_output(output_id: &str, safe_user: &SafeUser) -> Result<Output, Error> {
    let path = format!("/safe/outputs/{output_id}");
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    let parsed: ApiResponse<Output> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain output data".to_string()))
}

pub async fn fetch_safe_outputs(
    user_id: &str,
    output_ids: &[String],
    safe_user: &SafeUser,
) -> Result<Vec<Output>, Error> {
    let path = "/safe/outputs/fetch";
    let data = OutputFetchRequest {
        user_id,
        ids: output_ids,
    };
    let data_str = serde_json::to_string(&data)?;
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<Vec<Output>> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        if api_error.code == 404 {
            let mut outputs = Vec::with_capacity(output_ids.len());
            for output_id in output_ids {
                outputs.push(get_output(output_id, safe_user).await?);
            }
            return Ok(outputs);
        }
        return Err(Error::Api(api_error));
    }
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain output data".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_query() {
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("members", "members")
            .append_pair("threshold", "1")
            .append_pair("limit", "500")
            .append_pair("state", OUTPUT_STATE_UNSPENT)
            .finish();
        assert!(query.contains("members=members"));
        assert!(query.contains("threshold=1"));
        assert!(query.contains("state=unspent"));
    }

    #[test]
    fn test_output_fetch_request_serialization() {
        let ids = vec!["output-1".to_string(), "output-2".to_string()];
        let request = OutputFetchRequest {
            user_id: "user-id",
            ids: &ids,
        };
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["user_id"], "user-id");
        assert_eq!(value["ids"][0], "output-1");
        assert_eq!(value["ids"][1], "output-2");
    }
}
