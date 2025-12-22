use serde::{Deserialize, Serialize};
use url::form_urlencoded;

use crate::{
    auth::sign_authentication_token,
    error::Error,
    models::{CollectibleOutput, CollectibleToken},
    request::{ApiResponse, request},
    safe::SafeUser,
};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CollectibleOutputQuery {
    pub members: String,
    pub threshold: u8,
    pub state: Option<String>,
    pub offset: Option<String>,
    pub limit: Option<u32>,
}

pub async fn read_collectible_token(
    token_id: &str,
    safe_user: &SafeUser,
) -> Result<CollectibleToken, Error> {
    let path = format!("/collectibles/tokens/{token_id}");
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    let parsed: ApiResponse<CollectibleToken> = serde_json::from_slice(&body)?;
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain collectible token".to_string())
    })
}

pub async fn list_collectible_outputs(
    query: &CollectibleOutputQuery,
    safe_user: &SafeUser,
) -> Result<Vec<CollectibleOutput>, Error> {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    serializer.append_pair("members", &query.members);
    serializer.append_pair("threshold", &query.threshold.to_string());
    serializer.append_pair("limit", &query.limit.unwrap_or(100).to_string());
    if let Some(state) = &query.state {
        serializer.append_pair("state", state);
    }
    if let Some(offset) = &query.offset {
        serializer.append_pair("offset", offset);
    }

    let query_str = serializer.finish();
    let path = format!("/collectibles/outputs?{query_str}");
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    let parsed: ApiResponse<Vec<CollectibleOutput>> = serde_json::from_slice(&body)?;
    parsed.data.ok_or_else(|| {
        Error::DataNotFound("API response did not contain collectible outputs".to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collectible_output_query_serialization() {
        let query = CollectibleOutputQuery {
            members: "members".to_string(),
            threshold: 1,
            state: Some("unspent".to_string()),
            offset: Some("offset".to_string()),
            limit: Some(50),
        };
        let mut serializer = form_urlencoded::Serializer::new(String::new());
        serializer.append_pair("members", &query.members);
        serializer.append_pair("threshold", &query.threshold.to_string());
        serializer.append_pair("limit", &query.limit.unwrap().to_string());
        serializer.append_pair("state", query.state.as_ref().unwrap());
        serializer.append_pair("offset", query.offset.as_ref().unwrap());
        let query_str = serializer.finish();
        assert!(query_str.contains("members=members"));
        assert!(query_str.contains("threshold=1"));
        assert!(query_str.contains("state=unspent"));
    }
}
