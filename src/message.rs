use aes::Aes256;
use aes_gcm::{
    Aes128Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use cbc::{Decryptor, Encryptor};
use cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use ed25519_dalek::SigningKey;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::sign_authentication_token,
    error::Error,
    pin::{private_key_to_curve25519, public_key_to_curve25519},
    request::{ApiResponse, request},
    safe::SafeUser,
    session::{UserSession, fetch_user_sessions},
    utils::unique_conversation_id,
};

pub const MESSAGE_STATUS_SENT: &str = "SENT";
pub const MESSAGE_STATUS_DELIVERED: &str = "DELIVERED";
pub const MESSAGE_STATUS_READ: &str = "READ";

pub const MESSAGE_CATEGORY_PLAIN_TEXT: &str = "PLAIN_TEXT";
pub const MESSAGE_CATEGORY_PLAIN_AUDIO: &str = "PLAIN_AUDIO";
pub const MESSAGE_CATEGORY_PLAIN_POST: &str = "PLAIN_POST";
pub const MESSAGE_CATEGORY_PLAIN_IMAGE: &str = "PLAIN_IMAGE";
pub const MESSAGE_CATEGORY_PLAIN_DATA: &str = "PLAIN_DATA";
pub const MESSAGE_CATEGORY_PLAIN_STICKER: &str = "PLAIN_STICKER";
pub const MESSAGE_CATEGORY_PLAIN_LIVE: &str = "PLAIN_LIVE";
pub const MESSAGE_CATEGORY_PLAIN_LOCATION: &str = "PLAIN_LOCATION";
pub const MESSAGE_CATEGORY_PLAIN_VIDEO: &str = "PLAIN_VIDEO";
pub const MESSAGE_CATEGORY_PLAIN_CONTACT: &str = "PLAIN_CONTACT";
pub const MESSAGE_CATEGORY_PLAIN_TRANSCRIPT: &str = "PLAIN_TRANSCRIPT";
pub const MESSAGE_CATEGORY_SYSTEM_CONVERSATION: &str = "SYSTEM_CONVERSATION";
pub const MESSAGE_CATEGORY_SYSTEM_ACCOUNT_SNAPSHOT: &str = "SYSTEM_ACCOUNT_SNAPSHOT";
pub const MESSAGE_CATEGORY_SYSTEM_SAFE_SNAPSHOT: &str = "SYSTEM_SAFE_SNAPSHOT";
pub const MESSAGE_CATEGORY_SYSTEM_SAFE_INSCRIPTION: &str = "SYSTEM_SAFE_INSCRIPTION";
pub const MESSAGE_CATEGORY_MESSAGE_RECALL: &str = "MESSAGE_RECALL";
pub const MESSAGE_CATEGORY_MESSAGE_PIN: &str = "MESSAGE_PIN";
pub const MESSAGE_CATEGORY_APP_BUTTON_GROUP: &str = "APP_BUTTON_GROUP";
pub const MESSAGE_CATEGORY_APP_CARD: &str = "APP_CARD";

pub const MAX_APP_BUTTONS: usize = 18;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct MessageRequest {
    pub conversation_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient_id: Option<String>,
    pub message_id: String,
    pub category: String,
    pub data_base64: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub representative_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub silent: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_in: Option<u64>,
}

impl MessageRequest {
    pub fn new_base64(
        conversation_id: impl Into<String>,
        recipient_id: Option<String>,
        category: impl Into<String>,
        data_base64: impl Into<String>,
    ) -> Self {
        Self {
            conversation_id: conversation_id.into(),
            recipient_id,
            message_id: Uuid::new_v4().to_string(),
            category: category.into(),
            data_base64: data_base64.into(),
            representative_id: None,
            quote_message_id: None,
            silent: None,
            expire_in: None,
        }
    }

    pub fn new_data(
        conversation_id: impl Into<String>,
        recipient_id: Option<String>,
        category: impl Into<String>,
        data: impl AsRef<[u8]>,
    ) -> Self {
        Self::new_base64(
            conversation_id,
            recipient_id,
            category,
            encode_message_data(data),
        )
    }

    pub fn new_json<T>(
        conversation_id: impl Into<String>,
        recipient_id: Option<String>,
        category: impl Into<String>,
        data: &T,
    ) -> Result<Self, Error>
    where
        T: Serialize + ?Sized,
    {
        Ok(Self::new_base64(
            conversation_id,
            recipient_id,
            category,
            encode_message_json(data)?,
        ))
    }

    pub fn direct_base64(
        sender_user_id: &str,
        recipient_id: &str,
        category: impl Into<String>,
        data_base64: impl Into<String>,
    ) -> Self {
        Self::new_base64(
            unique_conversation_id(sender_user_id, recipient_id),
            Some(recipient_id.to_string()),
            category,
            data_base64,
        )
    }

    pub fn direct_data(
        sender_user_id: &str,
        recipient_id: &str,
        category: impl Into<String>,
        data: impl AsRef<[u8]>,
    ) -> Self {
        Self::direct_base64(
            sender_user_id,
            recipient_id,
            category,
            encode_message_data(data),
        )
    }

    pub fn direct_json<T>(
        sender_user_id: &str,
        recipient_id: &str,
        category: impl Into<String>,
        data: &T,
    ) -> Result<Self, Error>
    where
        T: Serialize + ?Sized,
    {
        Ok(Self::direct_base64(
            sender_user_id,
            recipient_id,
            category,
            encode_message_json(data)?,
        ))
    }

    pub fn with_message_id(mut self, message_id: impl Into<String>) -> Self {
        self.message_id = message_id.into();
        self
    }

    pub fn with_representative_id(mut self, representative_id: impl Into<String>) -> Self {
        self.representative_id = Some(representative_id.into());
        self
    }

    pub fn with_quote_message_id(mut self, quote_message_id: impl Into<String>) -> Self {
        self.quote_message_id = Some(quote_message_id.into());
        self
    }

    pub fn with_silent(mut self, silent: bool) -> Self {
        self.silent = Some(silent);
        self
    }

    pub fn with_expire_in(mut self, expire_in: u64) -> Self {
        self.expire_in = Some(expire_in);
        self
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct StickerMessagePayload {
    pub sticker_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ImageMessagePayload {
    pub attachment_id: String,
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct AudioMessagePayload {
    pub attachment_id: String,
    pub mime_type: String,
    pub size: u64,
    pub duration: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wave_form: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct VideoMessagePayload {
    pub attachment_id: String,
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
    pub size: u64,
    pub duration: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ContactMessagePayload {
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct AppButtonPayload {
    pub label: String,
    pub action: String,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct AppCardMessagePayload {
    pub app_id: String,
    pub icon_url: String,
    pub title: String,
    pub description: String,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<AppButtonPayload>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shareable: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct FileMessagePayload {
    pub attachment_id: String,
    pub mime_type: String,
    pub size: u64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct LiveMessagePayload {
    pub width: u32,
    pub height: u32,
    pub thumb_url: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shareable: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LocationMessagePayload {
    pub longitude: f64,
    pub latitude: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TransferMessagePayload {
    #[serde(rename = "type")]
    pub transfer_type: String,
    pub snapshot_id: String,
    pub opponent_id: String,
    pub asset_id: String,
    pub amount: String,
    pub trace_id: String,
    pub memo: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct RecallMessagePayload {
    pub message_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct MessageSession {
    pub user_id: String,
    pub session_id: String,
    pub public_key: String,
}

impl From<UserSession> for MessageSession {
    fn from(session: UserSession) -> Self {
        Self {
            user_id: session.user_id,
            session_id: session.session_id,
            public_key: session.public_key,
        }
    }
}

impl From<&UserSession> for MessageSession {
    fn from(session: &UserSession) -> Self {
        Self {
            user_id: session.user_id.clone(),
            session_id: session.session_id.clone(),
            public_key: session.public_key.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ReceiptAcknowledgementRequest {
    pub message_id: String,
    pub status: String,
}

pub fn encode_message_data(data: impl AsRef<[u8]>) -> String {
    URL_SAFE_NO_PAD.encode(data)
}

pub fn encode_message_json<T>(data: &T) -> Result<String, Error>
where
    T: Serialize + ?Sized,
{
    Ok(encode_message_data(serde_json::to_vec(data)?))
}

/// Encrypt message data with the Go SDK-compatible envelope.
///
/// This mirrors the Go SDK `EncryptMessageData` helper: it takes an existing
/// raw URL base64 message payload and returns an encrypted raw URL base64 binary
/// envelope. Standard Mixin Messenger clients do not automatically decrypt this
/// blob when it is sent as `PLAIN_TEXT`; only use it with recipients whose
/// client code explicitly calls `decrypt_message_data`.
pub fn encrypt_message_data_for_safe_user(
    data_base64: &str,
    sessions: &[MessageSession],
    safe_user: &SafeUser,
) -> Result<String, Error> {
    encrypt_message_data(data_base64, sessions, &safe_user.session_private_key)
}

/// Encrypt an existing raw URL base64 message payload.
///
/// See `encrypt_message_data_for_safe_user` for the delivery caveat.
pub fn encrypt_message_data(
    data_base64: &str,
    sessions: &[MessageSession],
    session_private_key: &str,
) -> Result<String, Error> {
    let data = URL_SAFE_NO_PAD
        .decode(data_base64)
        .map_err(|err| Error::Input(format!("invalid message data_base64: {err}")))?;
    encrypt_message_plaintext(&data, sessions, session_private_key)
}

/// Encrypt plaintext bytes into the Go SDK-compatible message envelope.
///
/// See `encrypt_message_data_for_safe_user` for the delivery caveat.
pub fn encrypt_message_plaintext_for_safe_user(
    data: impl AsRef<[u8]>,
    sessions: &[MessageSession],
    safe_user: &SafeUser,
) -> Result<String, Error> {
    encrypt_message_plaintext(data, sessions, &safe_user.session_private_key)
}

pub fn encrypt_message_plaintext(
    data: impl AsRef<[u8]>,
    sessions: &[MessageSession],
    session_private_key: &str,
) -> Result<String, Error> {
    encrypt_message_plaintext_with_rng(
        data.as_ref(),
        sessions,
        session_private_key,
        &mut rand::rngs::OsRng,
    )
}

pub fn decrypt_message_data_for_safe_user(
    encrypted_base64: &str,
    safe_user: &SafeUser,
) -> Result<Option<String>, Error> {
    decrypt_message_data(
        encrypted_base64,
        &safe_user.session_id,
        &safe_user.session_private_key,
    )
}

pub fn decrypt_message_data(
    encrypted_base64: &str,
    session_id: &str,
    session_private_key: &str,
) -> Result<Option<String>, Error> {
    let encrypted = URL_SAFE_NO_PAD
        .decode(encrypted_base64)
        .map_err(|err| Error::Input(format!("invalid encrypted message data: {err}")))?;
    let Some(plaintext) = decrypt_message_plaintext(&encrypted, session_id, session_private_key)?
    else {
        return Ok(None);
    };
    Ok(Some(encode_message_data(plaintext)))
}

pub fn decrypt_message_plaintext_for_safe_user(
    encrypted_base64: &str,
    safe_user: &SafeUser,
) -> Result<Option<Vec<u8>>, Error> {
    let encrypted = URL_SAFE_NO_PAD
        .decode(encrypted_base64)
        .map_err(|err| Error::Input(format!("invalid encrypted message data: {err}")))?;
    decrypt_message_plaintext(
        &encrypted,
        &safe_user.session_id,
        &safe_user.session_private_key,
    )
}

fn encrypt_message_plaintext_with_rng(
    data: &[u8],
    sessions: &[MessageSession],
    session_private_key: &str,
    rng: &mut impl RngCore,
) -> Result<String, Error> {
    if sessions.len() > u16::MAX as usize {
        return Err(Error::Input("too many message sessions".to_string()));
    }

    let seed = parse_ed25519_seed(session_private_key)?;
    let signing_key = SigningKey::from_bytes(&seed);
    let sender_ed_public = signing_key.verifying_key().to_bytes();
    let sender_curve_public = public_key_to_curve25519(&sender_ed_public)?;
    let sender_curve_private = private_key_to_curve25519(&seed);

    let mut message_key = [0u8; 16];
    rng.fill_bytes(&mut message_key);
    let mut nonce = [0u8; 12];
    rng.fill_bytes(&mut nonce);

    let cipher = Aes128Gcm::new_from_slice(&message_key)
        .map_err(|err| Error::Server(format!("AES-GCM key error: {err}")))?;
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), data)
        .map_err(|err| Error::Server(format!("AES-GCM encrypt error: {err}")))?;

    let mut result = Vec::with_capacity(1 + 2 + 32 + sessions.len() * 64 + 12 + ciphertext.len());
    result.push(1);
    result.extend_from_slice(&(sessions.len() as u16).to_le_bytes());
    result.extend_from_slice(&sender_curve_public);

    for session in sessions {
        let session_id = Uuid::parse_str(&session.session_id)
            .map_err(|err| Error::Input(format!("invalid session id: {err}")))?;
        let peer_public = decode_curve_public_key(&session.public_key)?;
        let shared = x25519_dalek::x25519(sender_curve_private, peer_public);

        let mut iv = [0u8; 16];
        rng.fill_bytes(&mut iv);
        let mut key_buf = message_key.to_vec();
        key_buf.resize(message_key.len() + 16, 0);
        let encrypted_key = Encryptor::<Aes256>::new_from_slices(&shared, &iv)
            .map_err(|err| Error::Server(format!("AES-CBC key error: {err}")))?
            .encrypt_padded_mut::<Pkcs7>(&mut key_buf, message_key.len())
            .map_err(|err| Error::Server(format!("AES-CBC encrypt error: {err}")))?;

        result.extend_from_slice(session_id.as_bytes());
        result.extend_from_slice(&iv);
        result.extend_from_slice(encrypted_key);
    }

    result.extend_from_slice(&nonce);
    result.extend_from_slice(&ciphertext);
    Ok(URL_SAFE_NO_PAD.encode(result))
}

fn decrypt_message_plaintext(
    encrypted: &[u8],
    session_id: &str,
    session_private_key: &str,
) -> Result<Option<Vec<u8>>, Error> {
    let seed = parse_ed25519_seed(session_private_key)?;
    let session_id = Uuid::parse_str(session_id)
        .map_err(|err| Error::Input(format!("invalid session id: {err}")))?;

    if encrypted.len() < 1 + 2 + 32 + 12 {
        return Err(Error::Input(
            "encrypted message data is too short".to_string(),
        ));
    }
    if encrypted[0] != 1 {
        return Err(Error::Input(format!(
            "unsupported encrypted message version: {}",
            encrypted[0]
        )));
    }

    let session_count = u16::from_le_bytes([encrypted[1], encrypted[2]]) as usize;
    let prefix_size = 35 + session_count * 64;
    if encrypted.len() < prefix_size + 12 {
        return Err(Error::Input(
            "encrypted message data is truncated".to_string(),
        ));
    }

    let sender_public: [u8; 32] = encrypted[3..35]
        .try_into()
        .map_err(|_| Error::Input("invalid sender curve public key".to_string()))?;
    let private = private_key_to_curve25519(&seed);
    let shared = x25519_dalek::x25519(private, sender_public);

    let mut message_key = None;
    for index in 0..session_count {
        let offset = 35 + index * 64;
        let block_session_id = Uuid::from_bytes(
            encrypted[offset..offset + 16]
                .try_into()
                .map_err(|_| Error::Input("invalid encrypted session id".to_string()))?,
        );
        if block_session_id != session_id {
            continue;
        }

        let iv = &encrypted[offset + 16..offset + 32];
        let encrypted_key = &encrypted[offset + 32..offset + 64];
        let mut key_buf = encrypted_key.to_vec();
        let decrypted_key = Decryptor::<Aes256>::new_from_slices(&shared, iv)
            .map_err(|err| Error::Server(format!("AES-CBC key error: {err}")))?
            .decrypt_padded_mut::<Pkcs7>(&mut key_buf)
            .map_err(|err| Error::Server(format!("AES-CBC decrypt error: {err}")))?;
        if decrypted_key.len() < 16 {
            return Err(Error::Input(
                "decrypted message key is too short".to_string(),
            ));
        }
        let mut key = [0u8; 16];
        key.copy_from_slice(&decrypted_key[..16]);
        message_key = Some(key);
        break;
    }

    let Some(message_key) = message_key else {
        return Ok(None);
    };

    let nonce = &encrypted[prefix_size..prefix_size + 12];
    let ciphertext = &encrypted[prefix_size + 12..];
    let cipher = Aes128Gcm::new_from_slice(&message_key)
        .map_err(|err| Error::Server(format!("AES-GCM key error: {err}")))?;
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|err| Error::Server(format!("AES-GCM decrypt error: {err}")))?;
    Ok(Some(plaintext))
}

fn parse_ed25519_seed(private_key: &str) -> Result<[u8; 32], Error> {
    if let Ok(bytes) = hex::decode(private_key) {
        return seed_from_private_key_bytes(&bytes);
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(private_key)
        .map_err(|err| Error::Input(format!("invalid ed25519 private key: {err}")))?;
    seed_from_private_key_bytes(&bytes)
}

fn seed_from_private_key_bytes(bytes: &[u8]) -> Result<[u8; 32], Error> {
    match bytes.len() {
        32 | 64 => bytes[..32]
            .try_into()
            .map_err(|_| Error::Input("invalid ed25519 private key length".to_string())),
        len => Err(Error::Input(format!(
            "invalid ed25519 private key length: {len}"
        ))),
    }
}

fn decode_curve_public_key(public_key: &str) -> Result<[u8; 32], Error> {
    let bytes = URL_SAFE_NO_PAD
        .decode(public_key)
        .map_err(|err| Error::Input(format!("invalid session public key: {err}")))?;
    bytes
        .try_into()
        .map_err(|_| Error::Input("invalid session public key length".to_string()))
}

pub fn data_message_request(
    sender_user_id: &str,
    recipient_id: &str,
    category: &str,
    data: impl AsRef<[u8]>,
) -> MessageRequest {
    MessageRequest::direct_data(sender_user_id, recipient_id, category, data)
}

pub fn json_message_request<T>(
    sender_user_id: &str,
    recipient_id: &str,
    category: &str,
    payload: &T,
) -> Result<MessageRequest, Error>
where
    T: Serialize + ?Sized,
{
    MessageRequest::direct_json(sender_user_id, recipient_id, category, payload)
}

pub fn text_message_request(
    sender_user_id: &str,
    recipient_id: &str,
    text: &str,
) -> MessageRequest {
    data_message_request(
        sender_user_id,
        recipient_id,
        MESSAGE_CATEGORY_PLAIN_TEXT,
        text,
    )
}

/// Build a message request whose `data_base64` is an encrypted binary blob.
///
/// This matches the Go SDK pattern of calling `EncryptMessageData` and passing
/// the result as `DataBase64` to `PostMessage`.
///
/// This is not suitable for normal Messenger UI recipients unless their client
/// explicitly knows how to decrypt the Mixin bot encrypted-message envelope.
pub fn encrypted_data_message_request(
    sender_user_id: &str,
    recipient_id: &str,
    category: &str,
    data: impl AsRef<[u8]>,
    sessions: &[MessageSession],
    safe_user: &SafeUser,
) -> Result<MessageRequest, Error> {
    Ok(MessageRequest::direct_base64(
        sender_user_id,
        recipient_id,
        category,
        encrypt_message_plaintext_for_safe_user(data, sessions, safe_user)?,
    ))
}

/// Build an encrypted `PLAIN_TEXT` request for custom decrypting clients.
///
/// Normal Messenger UI recipients will see unreadable text.
pub fn encrypted_text_message_request(
    sender_user_id: &str,
    recipient_id: &str,
    text: &str,
    sessions: &[MessageSession],
    safe_user: &SafeUser,
) -> Result<MessageRequest, Error> {
    encrypted_data_message_request(
        sender_user_id,
        recipient_id,
        MESSAGE_CATEGORY_PLAIN_TEXT,
        text,
        sessions,
        safe_user,
    )
}

pub fn post_message_request(
    sender_user_id: &str,
    recipient_id: &str,
    text: &str,
) -> MessageRequest {
    data_message_request(
        sender_user_id,
        recipient_id,
        MESSAGE_CATEGORY_PLAIN_POST,
        text,
    )
}

pub fn sticker_message_request(
    sender_user_id: &str,
    recipient_id: &str,
    sticker: &StickerMessagePayload,
) -> Result<MessageRequest, Error> {
    json_message_request(
        sender_user_id,
        recipient_id,
        MESSAGE_CATEGORY_PLAIN_STICKER,
        sticker,
    )
}

pub fn image_message_request(
    sender_user_id: &str,
    recipient_id: &str,
    image: &ImageMessagePayload,
) -> Result<MessageRequest, Error> {
    json_message_request(
        sender_user_id,
        recipient_id,
        MESSAGE_CATEGORY_PLAIN_IMAGE,
        image,
    )
}

pub fn audio_message_request(
    sender_user_id: &str,
    recipient_id: &str,
    audio: &AudioMessagePayload,
) -> Result<MessageRequest, Error> {
    json_message_request(
        sender_user_id,
        recipient_id,
        MESSAGE_CATEGORY_PLAIN_AUDIO,
        audio,
    )
}

pub fn video_message_request(
    sender_user_id: &str,
    recipient_id: &str,
    video: &VideoMessagePayload,
) -> Result<MessageRequest, Error> {
    json_message_request(
        sender_user_id,
        recipient_id,
        MESSAGE_CATEGORY_PLAIN_VIDEO,
        video,
    )
}

pub fn contact_message_request(
    sender_user_id: &str,
    recipient_id: &str,
    contact: &ContactMessagePayload,
) -> Result<MessageRequest, Error> {
    json_message_request(
        sender_user_id,
        recipient_id,
        MESSAGE_CATEGORY_PLAIN_CONTACT,
        contact,
    )
}

pub fn app_card_message_request(
    sender_user_id: &str,
    recipient_id: &str,
    app_card: &AppCardMessagePayload,
) -> Result<MessageRequest, Error> {
    if let Some(actions) = &app_card.actions {
        validate_app_buttons(actions)?;
    }
    json_message_request(
        sender_user_id,
        recipient_id,
        MESSAGE_CATEGORY_APP_CARD,
        app_card,
    )
}

pub fn file_message_request(
    sender_user_id: &str,
    recipient_id: &str,
    file: &FileMessagePayload,
) -> Result<MessageRequest, Error> {
    json_message_request(
        sender_user_id,
        recipient_id,
        MESSAGE_CATEGORY_PLAIN_DATA,
        file,
    )
}

pub fn live_message_request(
    sender_user_id: &str,
    recipient_id: &str,
    live: &LiveMessagePayload,
) -> Result<MessageRequest, Error> {
    json_message_request(
        sender_user_id,
        recipient_id,
        MESSAGE_CATEGORY_PLAIN_LIVE,
        live,
    )
}

pub fn location_message_request(
    sender_user_id: &str,
    recipient_id: &str,
    location: &LocationMessagePayload,
) -> Result<MessageRequest, Error> {
    json_message_request(
        sender_user_id,
        recipient_id,
        MESSAGE_CATEGORY_PLAIN_LOCATION,
        location,
    )
}

pub fn app_button_group_message_request(
    sender_user_id: &str,
    recipient_id: &str,
    buttons: &[AppButtonPayload],
) -> Result<MessageRequest, Error> {
    validate_app_buttons(buttons)?;
    json_message_request(
        sender_user_id,
        recipient_id,
        MESSAGE_CATEGORY_APP_BUTTON_GROUP,
        buttons,
    )
}

pub fn transfer_message_request(
    sender_user_id: &str,
    recipient_id: &str,
    transfer: &TransferMessagePayload,
) -> Result<MessageRequest, Error> {
    json_message_request(
        sender_user_id,
        recipient_id,
        MESSAGE_CATEGORY_SYSTEM_ACCOUNT_SNAPSHOT,
        transfer,
    )
}

pub fn recall_message_request(
    sender_user_id: &str,
    recipient_id: &str,
    recall: &RecallMessagePayload,
) -> Result<MessageRequest, Error> {
    json_message_request(
        sender_user_id,
        recipient_id,
        MESSAGE_CATEGORY_MESSAGE_RECALL,
        recall,
    )
}

pub fn acknowledgement_request(
    message_id: impl Into<String>,
    status: impl Into<String>,
) -> ReceiptAcknowledgementRequest {
    ReceiptAcknowledgementRequest {
        message_id: message_id.into(),
        status: status.into(),
    }
}

pub fn read_acknowledgement_request(
    message_id: impl Into<String>,
) -> ReceiptAcknowledgementRequest {
    acknowledgement_request(message_id, MESSAGE_STATUS_READ)
}

pub async fn post_messages(messages: &[MessageRequest], safe_user: &SafeUser) -> Result<(), Error> {
    let data_str = serde_json::to_string(messages)?;
    let path = "/messages";
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<serde_json::Value> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    Ok(())
}

pub async fn post_message(message: MessageRequest, safe_user: &SafeUser) -> Result<(), Error> {
    post_messages(&[message], safe_user).await
}

pub async fn send_message_request(
    message: MessageRequest,
    safe_user: &SafeUser,
) -> Result<MessageRequest, Error> {
    let sent = message.clone();
    post_message(message, safe_user).await?;
    Ok(sent)
}

pub async fn send_data_message(
    recipient_id: &str,
    category: &str,
    data: impl AsRef<[u8]>,
    safe_user: &SafeUser,
) -> Result<MessageRequest, Error> {
    let message = data_message_request(&safe_user.user_id, recipient_id, category, data);
    send_message_request(message, safe_user).await
}

pub async fn send_json_message<T>(
    recipient_id: &str,
    category: &str,
    payload: &T,
    safe_user: &SafeUser,
) -> Result<MessageRequest, Error>
where
    T: Serialize + ?Sized,
{
    let message = json_message_request(&safe_user.user_id, recipient_id, category, payload)?;
    send_message_request(message, safe_user).await
}

pub async fn send_text_message(
    recipient_id: &str,
    text: &str,
    safe_user: &SafeUser,
) -> Result<MessageRequest, Error> {
    let message = text_message_request(&safe_user.user_id, recipient_id, text);
    send_message_request(message, safe_user).await
}

/// Send an encrypted text payload to a custom decrypting recipient.
///
/// Normal Messenger UI recipients will see unreadable text.
pub async fn send_encrypted_text_message(
    recipient_id: &str,
    text: &str,
    sessions: &[MessageSession],
    safe_user: &SafeUser,
) -> Result<MessageRequest, Error> {
    let message = encrypted_text_message_request(
        &safe_user.user_id,
        recipient_id,
        text,
        sessions,
        safe_user,
    )?;
    send_message_request(message, safe_user).await
}

/// Fetch recipient sessions and send an encrypted text payload.
///
/// Normal Messenger UI recipients will see unreadable text.
pub async fn send_encrypted_text_message_to_user(
    recipient_id: &str,
    text: &str,
    safe_user: &SafeUser,
) -> Result<MessageRequest, Error> {
    let sessions = fetch_user_sessions(&[recipient_id.to_string()], safe_user).await?;
    let sessions: Vec<MessageSession> = sessions.iter().map(MessageSession::from).collect();
    send_encrypted_text_message(recipient_id, text, &sessions, safe_user).await
}

pub async fn send_post_message(
    recipient_id: &str,
    text: &str,
    safe_user: &SafeUser,
) -> Result<MessageRequest, Error> {
    let message = post_message_request(&safe_user.user_id, recipient_id, text);
    send_message_request(message, safe_user).await
}

pub async fn send_sticker_message(
    recipient_id: &str,
    sticker: &StickerMessagePayload,
    safe_user: &SafeUser,
) -> Result<MessageRequest, Error> {
    let message = sticker_message_request(&safe_user.user_id, recipient_id, sticker)?;
    send_message_request(message, safe_user).await
}

pub async fn send_image_message(
    recipient_id: &str,
    image: &ImageMessagePayload,
    safe_user: &SafeUser,
) -> Result<MessageRequest, Error> {
    let message = image_message_request(&safe_user.user_id, recipient_id, image)?;
    send_message_request(message, safe_user).await
}

pub async fn send_audio_message(
    recipient_id: &str,
    audio: &AudioMessagePayload,
    safe_user: &SafeUser,
) -> Result<MessageRequest, Error> {
    let message = audio_message_request(&safe_user.user_id, recipient_id, audio)?;
    send_message_request(message, safe_user).await
}

pub async fn send_video_message(
    recipient_id: &str,
    video: &VideoMessagePayload,
    safe_user: &SafeUser,
) -> Result<MessageRequest, Error> {
    let message = video_message_request(&safe_user.user_id, recipient_id, video)?;
    send_message_request(message, safe_user).await
}

pub async fn send_contact_message(
    recipient_id: &str,
    contact: &ContactMessagePayload,
    safe_user: &SafeUser,
) -> Result<MessageRequest, Error> {
    let message = contact_message_request(&safe_user.user_id, recipient_id, contact)?;
    send_message_request(message, safe_user).await
}

pub async fn send_app_card_message(
    recipient_id: &str,
    app_card: &AppCardMessagePayload,
    safe_user: &SafeUser,
) -> Result<MessageRequest, Error> {
    let message = app_card_message_request(&safe_user.user_id, recipient_id, app_card)?;
    send_message_request(message, safe_user).await
}

pub async fn send_file_message(
    recipient_id: &str,
    file: &FileMessagePayload,
    safe_user: &SafeUser,
) -> Result<MessageRequest, Error> {
    let message = file_message_request(&safe_user.user_id, recipient_id, file)?;
    send_message_request(message, safe_user).await
}

pub async fn send_live_message(
    recipient_id: &str,
    live: &LiveMessagePayload,
    safe_user: &SafeUser,
) -> Result<MessageRequest, Error> {
    let message = live_message_request(&safe_user.user_id, recipient_id, live)?;
    send_message_request(message, safe_user).await
}

pub async fn send_location_message(
    recipient_id: &str,
    location: &LocationMessagePayload,
    safe_user: &SafeUser,
) -> Result<MessageRequest, Error> {
    let message = location_message_request(&safe_user.user_id, recipient_id, location)?;
    send_message_request(message, safe_user).await
}

pub async fn send_app_button_group_message(
    recipient_id: &str,
    buttons: &[AppButtonPayload],
    safe_user: &SafeUser,
) -> Result<MessageRequest, Error> {
    let message = app_button_group_message_request(&safe_user.user_id, recipient_id, buttons)?;
    send_message_request(message, safe_user).await
}

pub async fn send_transfer_message(
    recipient_id: &str,
    transfer: &TransferMessagePayload,
    safe_user: &SafeUser,
) -> Result<MessageRequest, Error> {
    let message = transfer_message_request(&safe_user.user_id, recipient_id, transfer)?;
    send_message_request(message, safe_user).await
}

pub async fn send_recall_message(
    recipient_id: &str,
    recall: &RecallMessagePayload,
    safe_user: &SafeUser,
) -> Result<MessageRequest, Error> {
    let message = recall_message_request(&safe_user.user_id, recipient_id, recall)?;
    send_message_request(message, safe_user).await
}

pub async fn post_acknowledgements(
    requests: &[ReceiptAcknowledgementRequest],
    safe_user: &SafeUser,
) -> Result<(), Error> {
    let data_str = serde_json::to_string(requests)?;
    let path = "/acknowledgements";
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<serde_json::Value> = serde_json::from_slice(&body)?;
    if let Some(api_error) = parsed.error {
        return Err(Error::Api(api_error));
    }
    Ok(())
}

pub async fn post_acknowledgement(
    request: ReceiptAcknowledgementRequest,
    safe_user: &SafeUser,
) -> Result<(), Error> {
    let requests = [request];
    post_acknowledgements(&requests, safe_user).await
}

pub async fn acknowledge_message(
    message_id: &str,
    status: &str,
    safe_user: &SafeUser,
) -> Result<(), Error> {
    post_acknowledgement(acknowledgement_request(message_id, status), safe_user).await
}

pub async fn mark_message_read(message_id: &str, safe_user: &SafeUser) -> Result<(), Error> {
    acknowledge_message(message_id, MESSAGE_STATUS_READ, safe_user).await
}

fn validate_app_buttons(buttons: &[AppButtonPayload]) -> Result<(), Error> {
    if buttons.len() > MAX_APP_BUTTONS {
        return Err(Error::Input(format!(
            "too many app buttons: max {MAX_APP_BUTTONS}, got {}",
            buttons.len()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use base64::Engine as _;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use ed25519_dalek::SigningKey;
    use rand::RngCore;

    use super::*;

    struct FixedRng {
        bytes: Vec<u8>,
        offset: usize,
    }

    impl FixedRng {
        fn new(bytes: Vec<u8>) -> Self {
            Self { bytes, offset: 0 }
        }
    }

    impl RngCore for FixedRng {
        fn next_u32(&mut self) -> u32 {
            let mut bytes = [0u8; 4];
            self.fill_bytes(&mut bytes);
            u32::from_le_bytes(bytes)
        }

        fn next_u64(&mut self) -> u64 {
            let mut bytes = [0u8; 8];
            self.fill_bytes(&mut bytes);
            u64::from_le_bytes(bytes)
        }

        fn fill_bytes(&mut self, dest: &mut [u8]) {
            let end = self.offset + dest.len();
            dest.copy_from_slice(&self.bytes[self.offset..end]);
            self.offset = end;
        }

        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
            self.fill_bytes(dest);
            Ok(())
        }
    }

    fn test_session(seed_byte: u8, user_id: &str, session_id: &str) -> (String, MessageSession) {
        let seed = [seed_byte; 32];
        let signing_key = SigningKey::from_bytes(&seed);
        let ed_public = signing_key.verifying_key().to_bytes();
        let curve_public = public_key_to_curve25519(&ed_public).unwrap();
        (
            hex::encode(seed),
            MessageSession {
                user_id: user_id.to_string(),
                session_id: session_id.to_string(),
                public_key: URL_SAFE_NO_PAD.encode(curve_public),
            },
        )
    }

    #[test]
    fn test_message_request_serialization() {
        let request = MessageRequest::new_base64(
            "conversation-id",
            None,
            MESSAGE_CATEGORY_PLAIN_TEXT,
            "SGVsbG8",
        )
        .with_message_id("message-id")
        .with_quote_message_id("quote-id")
        .with_silent(true)
        .with_expire_in(60);

        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["conversation_id"], "conversation-id");
        assert_eq!(value["message_id"], "message-id");
        assert_eq!(value["category"], "PLAIN_TEXT");
        assert_eq!(value["data_base64"], "SGVsbG8");
        assert_eq!(value["quote_message_id"], "quote-id");
        assert_eq!(value["silent"], true);
        assert_eq!(value["expire_in"], 60);
        assert!(value.get("recipient_id").is_none());
    }

    #[test]
    fn test_text_message_request_uses_direct_conversation_and_raw_url_base64() {
        let request = text_message_request("test1", "test2", "Hello");

        assert_eq!(
            request.conversation_id,
            unique_conversation_id("test1", "test2")
        );
        assert_eq!(request.recipient_id.as_deref(), Some("test2"));
        assert_eq!(request.category, MESSAGE_CATEGORY_PLAIN_TEXT);
        assert_eq!(request.data_base64, "SGVsbG8");

        let decoded = URL_SAFE_NO_PAD.decode(request.data_base64).unwrap();
        assert_eq!(decoded, b"Hello");
    }

    #[test]
    fn test_app_card_message_request_serializes_json_payload() {
        let app_card = AppCardMessagePayload {
            app_id: "app-id".to_string(),
            icon_url: "https://example.com/icon.png".to_string(),
            title: "Title".to_string(),
            description: "Description".to_string(),
            action: "https://example.com".to_string(),
            cover_url: None,
            actions: None,
            shareable: Some(true),
        };

        let request = app_card_message_request("sender", "recipient", &app_card).unwrap();
        assert_eq!(request.category, MESSAGE_CATEGORY_APP_CARD);

        let decoded = URL_SAFE_NO_PAD.decode(request.data_base64).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&decoded).unwrap();
        assert_eq!(value["app_id"], "app-id");
        assert_eq!(value["icon_url"], "https://example.com/icon.png");
        assert_eq!(value["title"], "Title");
        assert_eq!(value["shareable"], true);
        assert!(value.get("cover_url").is_none());
    }

    #[test]
    fn test_app_button_group_rejects_too_many_buttons() {
        let button = AppButtonPayload {
            label: "Open".to_string(),
            action: "https://example.com".to_string(),
            color: "#00ff00".to_string(),
        };
        let buttons = vec![button; MAX_APP_BUTTONS + 1];

        let err = app_button_group_message_request("sender", "recipient", &buttons).unwrap_err();
        assert!(err.to_string().contains("too many app buttons"));
    }

    #[test]
    fn test_acknowledgement_serialization() {
        let ack = read_acknowledgement_request("message-id");
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&ack).unwrap()).unwrap();
        assert_eq!(value["message_id"], "message-id");
        assert_eq!(value["status"], "READ");
    }

    #[test]
    fn test_encrypt_decrypt_message_plaintext_roundtrip() {
        let (sender_private, _) = test_session(1, "sender", "00000000-0000-0000-0000-000000000001");
        let (recipient_private, recipient_session) =
            test_session(2, "recipient", "00000000-0000-0000-0000-000000000002");

        let encrypted = encrypt_message_plaintext(
            b"secret message",
            std::slice::from_ref(&recipient_session),
            &sender_private,
        )
        .unwrap();
        let decrypted = decrypt_message_data(
            &encrypted,
            &recipient_session.session_id,
            &recipient_private,
        )
        .unwrap()
        .unwrap();

        assert_eq!(decrypted, encode_message_data("secret message"));

        let payload = URL_SAFE_NO_PAD.decode(encrypted).unwrap();
        assert_eq!(payload[0], 1);
        assert_eq!(u16::from_le_bytes([payload[1], payload[2]]), 1);
        assert_eq!(
            payload.len(),
            1 + 2 + 32 + 64 + 12 + "secret message".len() + 16
        );
    }

    #[test]
    fn test_encrypt_decrypt_message_data_roundtrip() {
        let (sender_private, _) = test_session(3, "sender", "00000000-0000-0000-0000-000000000003");
        let (recipient_private, recipient_session) =
            test_session(4, "recipient", "00000000-0000-0000-0000-000000000004");

        let data_base64 = encode_message_data("hello");
        let encrypted = encrypt_message_data(
            &data_base64,
            std::slice::from_ref(&recipient_session),
            &sender_private,
        )
        .unwrap();
        let decrypted = decrypt_message_data(
            &encrypted,
            &recipient_session.session_id,
            &recipient_private,
        )
        .unwrap();

        assert_eq!(decrypted.as_deref(), Some(data_base64.as_str()));
    }

    #[test]
    fn test_encrypt_message_data_matches_go_fixture() {
        let sender_private_b64 = "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQGKiOPddAnxlf1S2y08ul1yymcJvx2UEhvzdIgBtA9vXA";
        let recipient_private_b64 = "AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgKBOXcOqH0XX1ajVGbDTH7My42KkbTuN6Jd9g9bj8mzlA";
        let expected = "AQEAGxtY3VDqFLYNoXt5DNAnVNlwybq4ZOuzwPMBb-UdP1cAAAAAAAAAAAAAAAAAAAACHB0eHyAhIiMkJSYnKCkqK7sw9GZd-HXqYA1lZX1fKF2nxW4ZsABVtE0oLMKLRLI0EBESExQVFhcYGRobrEtvw2BNPurqypdTWnv1QA3Tc4PC";
        let session = MessageSession {
            user_id: "recipient".to_string(),
            session_id: "00000000-0000-0000-0000-000000000002".to_string(),
            public_key: "YDRufJEaX2uhVBKRdMr-dbKUrDu9VUljL0jOxiZvhBA".to_string(),
        };
        let mut rng = FixedRng::new((0u8..44).collect());

        let encrypted = encrypt_message_plaintext_with_rng(
            b"hello",
            std::slice::from_ref(&session),
            sender_private_b64,
            &mut rng,
        )
        .unwrap();
        assert_eq!(encrypted, expected);

        let decrypted =
            decrypt_message_data(&encrypted, &session.session_id, recipient_private_b64)
                .unwrap()
                .unwrap();
        assert_eq!(decrypted, "aGVsbG8");
    }

    #[test]
    fn test_encrypted_text_message_request_builds_direct_message() {
        let (sender_private, _) = test_session(9, "sender", "00000000-0000-0000-0000-000000000009");
        let (recipient_private, recipient_session) =
            test_session(10, "recipient", "00000000-0000-0000-0000-000000000010");
        let safe_user = SafeUser {
            user_id: "sender".to_string(),
            session_id: "00000000-0000-0000-0000-000000000009".to_string(),
            session_private_key: sender_private,
            server_public_key: String::new(),
            spend_private_key: String::new(),
            is_spend_private_sum: false,
        };

        let request = encrypted_text_message_request(
            &safe_user.user_id,
            "recipient",
            "secret",
            std::slice::from_ref(&recipient_session),
            &safe_user,
        )
        .unwrap();

        assert_eq!(request.category, MESSAGE_CATEGORY_PLAIN_TEXT);
        assert_eq!(request.recipient_id.as_deref(), Some("recipient"));
        assert_eq!(
            request.conversation_id,
            unique_conversation_id("sender", "recipient")
        );
        assert_ne!(request.data_base64, encode_message_data("secret"));

        let decrypted = decrypt_message_data(
            &request.data_base64,
            &recipient_session.session_id,
            &recipient_private,
        )
        .unwrap()
        .unwrap();
        assert_eq!(decrypted, encode_message_data("secret"));
    }

    #[test]
    fn test_decrypt_message_data_returns_none_for_missing_session() {
        let (sender_private, _) = test_session(5, "sender", "00000000-0000-0000-0000-000000000005");
        let (_, recipient_session) =
            test_session(6, "recipient", "00000000-0000-0000-0000-000000000006");
        let (other_private, other_session) =
            test_session(7, "other", "00000000-0000-0000-0000-000000000007");

        let encrypted =
            encrypt_message_plaintext(b"secret", &[recipient_session], &sender_private).unwrap();
        let decrypted =
            decrypt_message_data(&encrypted, &other_session.session_id, &other_private).unwrap();

        assert!(decrypted.is_none());
    }

    #[test]
    fn test_parse_ed25519_seed_accepts_base64_seed() {
        let seed = [8u8; 32];
        let encoded = URL_SAFE_NO_PAD.encode(seed);
        assert_eq!(parse_ed25519_seed(&encoded).unwrap(), seed);
    }
}
