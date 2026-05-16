use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{
    auth::sign_authentication_token,
    error::Error,
    request::{ApiResponse, request},
    safe::SafeUser,
};

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq, Eq)]
pub struct Scheme {
    #[serde(default, rename = "type")]
    pub type_name: Option<String>,
    pub scheme_id: String,
    pub target: String,
}

pub async fn read_code<T>(code_id: &str) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    let path = format!("/codes/{code_id}");
    let body = request("GET", &path, &[], "").await?;
    parse_data(&body, "code")
}

pub async fn read_code_value(code_id: &str) -> Result<serde_json::Value, Error> {
    read_code(code_id).await
}

pub async fn create_scheme(target: &str, safe_user: &SafeUser) -> Result<Scheme, Error> {
    let path = "/schemes";
    let data = serde_json::json!({ "target": target });
    let data_str = data.to_string();
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;
    parse_data(&body, "scheme")
}

fn parse_data<T>(body: &[u8], label: &str) -> Result<T, Error>
where
    T: DeserializeOwned,
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
    fn test_scheme_deserialize() {
        let raw = r#"{
            "type": "scheme",
            "scheme_id": "scheme-id",
            "target": "mixin://users/user-id"
        }"#;
        let scheme: Scheme = serde_json::from_str(raw).expect("scheme");
        assert_eq!(scheme.type_name.as_deref(), Some("scheme"));
        assert_eq!(scheme.scheme_id, "scheme-id");
        assert_eq!(scheme.target, "mixin://users/user-id");
    }

    #[test]
    fn test_read_code_value_shape() {
        let body = br#"{"data":{"type":"user","user_id":"user-id"}}"#;
        let value: serde_json::Value = parse_data(body, "code").expect("code");
        assert_eq!(value["type"], "user");
        assert_eq!(value["user_id"], "user-id");
    }
}
