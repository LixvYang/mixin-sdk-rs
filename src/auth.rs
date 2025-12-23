use base64::{Engine as _, engine::general_purpose};
use ed25519_dalek::Signer;
use ed25519_dalek::SigningKey;
use hex;
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use serde_json;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use uuid::Uuid;

use crate::safe::SafeUser;

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aid: Option<String>,
    pub iat: u64,
    pub exp: u64,
    pub jti: String,
    pub sig: String,
    pub scp: String,
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid hex string: {0}")]
    Hex(#[from] hex::FromHexError),
    #[error("Invalid Ed25519 private key length")]
    InvalidKeyLength,
    #[error("JWT signing error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("Failed to get system time: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),
    #[error("Invalid private key bytes")]
    InvalidPrivateKeyBytes,
    #[error("Serde json error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub fn sign_authentication_token_without_body(
    method: &str,
    uri: &str,
    user: &SafeUser,
) -> Result<String, AuthError> {
    sign_authentication_token(method, uri, "", user)
}

pub fn sign_authentication_token(
    method: &str,
    uri: &str,
    body: &str,
    user: &SafeUser,
) -> Result<String, AuthError> {
    sign_authentication_token_with_request_id(method, uri, body, Uuid::new_v4().to_string(), user)
}

pub fn sign_authentication_token_with_request_id(
    method: &str,
    uri: &str,
    body: &str,
    request_id: String,
    user: &SafeUser,
) -> Result<String, AuthError> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let expire = now + (24 * 30 * 3 * 3600); // 3 months

    let mut hasher = Sha256::new();
    hasher.update((method.to_string() + uri + body).as_bytes());
    let sum = hasher.finalize();

    let claims = JwtClaims {
        uid: Some(user.user_id.clone()),
        sid: Some(user.session_id.clone()),
        iss: None,
        aid: None,
        iat: now,
        exp: expire,
        jti: request_id,
        sig: hex::encode(sum),
        scp: "FULL".to_string(),
    };

    let priv_key = hex::decode(&user.session_private_key)?;
    if priv_key.len() != 32 {
        return Err(AuthError::InvalidKeyLength);
    }

    let signing_key = SigningKey::from_bytes(
        &priv_key
            .try_into()
            .map_err(|_| AuthError::InvalidPrivateKeyBytes)?,
    );
    let claims_str = serde_json::to_string(&claims)?;

    let header_str = serde_json::to_string(&Header::new(jsonwebtoken::Algorithm::EdDSA))?;
    let b64_header = general_purpose::URL_SAFE_NO_PAD.encode(header_str);
    let b64_claims = general_purpose::URL_SAFE_NO_PAD.encode(claims_str);

    let message = format!("{}.{}", b64_header, b64_claims);
    let signature = signing_key.sign(message.as_bytes());

    let token = format!(
        "{}.{}",
        message,
        general_purpose::URL_SAFE_NO_PAD.encode(signature.to_bytes())
    );

    Ok(token)
}

#[allow(clippy::too_many_arguments)]
pub fn sign_oauth_access_token(
    app_id: &str,
    authorization_id: &str,
    private_key: &str,
    method: &str,
    uri: &str,
    body: &str,
    scope: &str,
    request_id: String,
) -> Result<String, AuthError> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let expire = now + (24 * 30 * 3 * 3600); // 3 months

    let mut hasher = Sha256::new();
    hasher.update((method.to_string() + uri + body).as_bytes());
    let sum = hasher.finalize();

    let claims = JwtClaims {
        uid: None,
        sid: None,
        iss: Some(app_id.to_string()),
        aid: Some(authorization_id.to_string()),
        iat: now,
        exp: expire,
        jti: request_id,
        sig: hex::encode(sum),
        scp: scope.to_string(),
    };

    let priv_key = hex::decode(private_key)?;
    if priv_key.len() != 32 {
        return Err(AuthError::InvalidKeyLength);
    }
    let signing_key = SigningKey::from_bytes(
        &priv_key
            .try_into()
            .map_err(|_| AuthError::InvalidPrivateKeyBytes)?,
    );

    let token = encode(
        &Header::new(jsonwebtoken::Algorithm::EdDSA),
        &claims,
        &EncodingKey::from_ed_der(&signing_key.to_keypair_bytes()),
    )?;

    Ok(token)
}

#[derive(Debug, Deserialize)]
pub struct OAuthTokenResponse {
    pub data: OAuthTokenData,
    pub error: Option<OAuthError>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthTokenData {
    pub scope: String,
    #[serde(rename = "access_token")]
    pub access_token: String,
    pub ed25519: Option<String>,
    #[serde(rename = "authorization_id")]
    pub authorization_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthError {
    pub code: i32,
    pub description: String,
}

pub async fn oauth_get_access_token(
    client_id: &str,
    client_secret: &str,
    authorization_code: &str,
    code_verifier: &str,
    ed25519: Option<&str>,
) -> Result<(String, String, Option<String>), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let mut params = serde_json::json!({
        "client_id": client_id,
        "client_secret": client_secret,
        "code": authorization_code,
        "code_verifier": code_verifier,
    });

    if let Some(ed25519) = ed25519 {
        params["ed25519"] = serde_json::Value::String(ed25519.to_string());
    }

    let response = client
        .post("https://api.mixin.one/oauth/token")
        .json(&params)
        .send()
        .await?;

    let oauth_response: OAuthTokenResponse = response.json().await?;

    if let Some(error) = oauth_response.error {
        return Err(format!("OAuth error: {}", error.description).into());
    }

    let data = oauth_response.data;
    if ed25519.is_none() {
        return Ok((data.access_token, data.scope, None));
    }

    Ok((
        data.ed25519.unwrap_or_default(),
        data.scope,
        data.authorization_id,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose;
    use sha2::{Digest, Sha256};

    fn test_user() -> SafeUser {
        SafeUser {
            user_id: "user-id".to_string(),
            session_id: "session-id".to_string(),
            session_private_key: "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
                .to_string(),
            server_public_key: "server-public-key".to_string(),
            spend_private_key: "spend-private-key".to_string(),
            is_spend_private_sum: false,
        }
    }

    #[test]
    fn test_sign_authentication_token_claims() {
        let user = test_user();
        let method = "POST";
        let uri = "/test";
        let body = "{\"hello\":\"world\"}";
        let request_id = "req-123";

        let token = sign_authentication_token_with_request_id(
            method,
            uri,
            body,
            request_id.to_string(),
            &user,
        )
        .expect("token");

        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);

        let claims_bytes = general_purpose::URL_SAFE_NO_PAD
            .decode(parts[1])
            .expect("claims b64");
        let claims: JwtClaims = serde_json::from_slice(&claims_bytes).expect("claims json");

        let mut hasher = Sha256::new();
        hasher.update((method.to_string() + uri + body).as_bytes());
        let expected_sig = hex::encode(hasher.finalize());

        assert_eq!(claims.uid.as_deref(), Some("user-id"));
        assert_eq!(claims.sid.as_deref(), Some("session-id"));
        assert_eq!(claims.jti, request_id);
        assert_eq!(claims.sig, expected_sig);
        assert_eq!(claims.scp, "FULL");
    }
}
