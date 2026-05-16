use serde::{Deserialize, Serialize};

use crate::{
    error::Error,
    request::{ApiResponse, request},
};

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq)]
pub struct Fiat {
    pub code: String,
    pub rate: f64,
}

pub async fn get_fiats() -> Result<Vec<Fiat>, Error> {
    let path = "/external/fiats";
    let body = request("GET", path, &[], "").await?;
    let parsed: ApiResponse<Vec<Fiat>> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("fiats".to_string()))
}

pub async fn fiats() -> Result<Vec<Fiat>, Error> {
    get_fiats().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fiat_deserialize() {
        let raw = r#"{"code":"USD","rate":1.0}"#;
        let fiat: Fiat = serde_json::from_str(raw).expect("fiat");
        assert_eq!(fiat.code, "USD");
        assert_eq!(fiat.rate, 1.0);
    }
}
