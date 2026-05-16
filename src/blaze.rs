use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::sync::Once;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{
        client::IntoClientRequest,
        http::{
            HeaderValue,
            header::{AUTHORIZATION, SEC_WEBSOCKET_PROTOCOL},
        },
        protocol::Message,
    },
};
use uuid::Uuid;

use crate::{
    auth::sign_authentication_token,
    error::Error,
    message::{
        MESSAGE_CATEGORY_SYSTEM_ACCOUNT_SNAPSHOT, MESSAGE_CATEGORY_SYSTEM_CONVERSATION,
        MESSAGE_CATEGORY_SYSTEM_SAFE_INSCRIPTION, MESSAGE_CATEGORY_SYSTEM_SAFE_SNAPSHOT,
        MESSAGE_STATUS_READ, MessageRequest,
    },
    request::{ApiError, get_blaze_uri},
    safe::SafeUser,
};

pub const BLAZE_SUBPROTOCOL: &str = "Mixin-Blaze-1";
pub const BLAZE_ACTION_CREATE_MESSAGE: &str = "CREATE_MESSAGE";
pub const BLAZE_ACTION_ACKNOWLEDGE_MESSAGE_RECEIPT: &str = "ACKNOWLEDGE_MESSAGE_RECEIPT";
pub const BLAZE_ACTION_LIST_PENDING_MESSAGES: &str = "LIST_PENDING_MESSAGES";

type BlazeWebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;
static RUSTLS_PROVIDER: Once = Once::new();

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlazeLoopOptions {
    pub sync_ack: bool,
    pub reconnect: bool,
    pub reconnect_delay: std::time::Duration,
    pub max_reconnects: Option<usize>,
}

impl Default for BlazeLoopOptions {
    fn default() -> Self {
        Self {
            sync_ack: true,
            reconnect: true,
            reconnect_delay: std::time::Duration::from_secs(5),
            max_reconnects: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlazeMessageKind {
    Message,
    AckReceipt,
    Transfer,
    Conversation,
    SafeSnapshot,
    SafeInscription,
}

#[async_trait::async_trait]
pub trait BlazeListener {
    async fn on_message(
        &mut self,
        _client: &mut BlazeClient,
        _message: MessageView,
        _client_user_id: &str,
    ) -> Result<(), Error>;

    async fn on_ack_receipt(
        &mut self,
        _client: &mut BlazeClient,
        _message: MessageView,
        _client_user_id: &str,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn on_transfer(
        &mut self,
        client: &mut BlazeClient,
        message: MessageView,
        client_user_id: &str,
    ) -> Result<(), Error> {
        self.on_message(client, message, client_user_id).await
    }

    async fn on_conversation(
        &mut self,
        client: &mut BlazeClient,
        message: MessageView,
        client_user_id: &str,
    ) -> Result<(), Error> {
        self.on_message(client, message, client_user_id).await
    }

    async fn on_safe_snapshot(
        &mut self,
        client: &mut BlazeClient,
        message: MessageView,
        client_user_id: &str,
    ) -> Result<(), Error> {
        self.on_message(client, message, client_user_id).await
    }

    async fn on_safe_inscription(
        &mut self,
        client: &mut BlazeClient,
        message: MessageView,
        client_user_id: &str,
    ) -> Result<(), Error> {
        self.on_message(client, message, client_user_id).await
    }

    async fn on_disconnect(&mut self, _error: &str, _attempt: usize) -> Result<(), Error> {
        Ok(())
    }

    fn sync_ack(&self) -> bool {
        true
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BlazeMessage {
    pub id: String,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
}

impl BlazeMessage {
    pub fn new(action: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            action: action.into(),
            params,
            data: None,
            error: None,
        }
    }

    pub fn message_view(&self) -> Result<Option<MessageView>, Error> {
        if self.action != BLAZE_ACTION_CREATE_MESSAGE
            && self.action != BLAZE_ACTION_ACKNOWLEDGE_MESSAGE_RECEIPT
        {
            return Ok(None);
        }

        let data = self
            .data
            .clone()
            .ok_or_else(|| Error::DataNotFound("blaze message data".to_string()))?;
        let mut message: MessageView = serde_json::from_value(data)?;
        if message.source.is_none() {
            message.source = Some(self.action.clone());
        }
        Ok(Some(message))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MessageView {
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub message_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub representative_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote_message_id: Option<String>,
    pub conversation_id: String,
    pub user_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub message_id: String,
    pub category: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    pub data_base64: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

impl MessageView {
    pub fn kind(&self) -> BlazeMessageKind {
        if self.source.as_deref() == Some(BLAZE_ACTION_ACKNOWLEDGE_MESSAGE_RECEIPT) {
            return BlazeMessageKind::AckReceipt;
        }
        match self.category.as_str() {
            MESSAGE_CATEGORY_SYSTEM_ACCOUNT_SNAPSHOT => BlazeMessageKind::Transfer,
            MESSAGE_CATEGORY_SYSTEM_CONVERSATION => BlazeMessageKind::Conversation,
            MESSAGE_CATEGORY_SYSTEM_SAFE_SNAPSHOT => BlazeMessageKind::SafeSnapshot,
            MESSAGE_CATEGORY_SYSTEM_SAFE_INSCRIPTION => BlazeMessageKind::SafeInscription,
            _ => BlazeMessageKind::Message,
        }
    }

    pub fn should_sync_ack(&self) -> bool {
        self.kind() != BlazeMessageKind::AckReceipt
    }

    pub fn data_bytes(&self) -> Result<Vec<u8>, Error> {
        URL_SAFE_NO_PAD
            .decode(&self.data_base64)
            .map_err(|err| Error::Input(format!("invalid message data_base64: {err}")))
    }

    pub fn data_text(&self) -> Result<String, Error> {
        String::from_utf8(self.data_bytes()?)
            .map_err(|err| Error::Input(format!("invalid message UTF-8 data: {err}")))
    }

    pub fn data_json<T: DeserializeOwned>(&self) -> Result<T, Error> {
        Ok(serde_json::from_slice(&self.data_bytes()?)?)
    }
}

pub struct BlazeClient {
    ws: BlazeWebSocket,
}

impl BlazeClient {
    pub async fn connect(safe_user: &SafeUser) -> Result<Self, Error> {
        ensure_rustls_provider();

        let token = sign_authentication_token("GET", "/", "", safe_user)?;
        let url = format!("wss://{}/", get_blaze_uri());
        let mut request = url
            .into_client_request()
            .map_err(|err| Error::Server(err.to_string()))?;
        request.headers_mut().insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}"))
                .map_err(|err| Error::Server(err.to_string()))?,
        );
        request.headers_mut().insert(
            SEC_WEBSOCKET_PROTOCOL,
            HeaderValue::from_static(BLAZE_SUBPROTOCOL),
        );

        let (ws, _) = connect_async(request)
            .await
            .map_err(|err| Error::Server(err.to_string()))?;
        Ok(Self { ws })
    }

    pub async fn send_raw(&mut self, message: &BlazeMessage) -> Result<(), Error> {
        let data = encode_blaze_message(message)?;
        self.ws
            .send(Message::Binary(data.into()))
            .await
            .map_err(|err| Error::Server(err.to_string()))
    }

    pub async fn send_action(
        &mut self,
        action: &str,
        params: Option<Value>,
    ) -> Result<String, Error> {
        let message = BlazeMessage::new(action, params);
        let id = message.id.clone();
        self.send_raw(&message).await?;
        Ok(id)
    }

    pub async fn list_pending_messages(&mut self) -> Result<String, Error> {
        self.send_action(BLAZE_ACTION_LIST_PENDING_MESSAGES, None)
            .await
    }

    pub async fn acknowledge_message(
        &mut self,
        message_id: &str,
        status: &str,
    ) -> Result<String, Error> {
        self.send_action(
            BLAZE_ACTION_ACKNOWLEDGE_MESSAGE_RECEIPT,
            Some(json!({
                "message_id": message_id,
                "status": status,
            })),
        )
        .await
    }

    pub async fn mark_message_read(&mut self, message_id: &str) -> Result<String, Error> {
        self.acknowledge_message(message_id, MESSAGE_STATUS_READ)
            .await
    }

    pub async fn send_message_request(
        &mut self,
        message: &MessageRequest,
    ) -> Result<String, Error> {
        self.send_action(
            BLAZE_ACTION_CREATE_MESSAGE,
            Some(serde_json::to_value(message)?),
        )
        .await
    }

    pub async fn next_blaze_message(&mut self) -> Result<BlazeMessage, Error> {
        while let Some(message) = self.ws.next().await {
            match message.map_err(|err| Error::Server(err.to_string()))? {
                Message::Binary(bytes) => return decode_blaze_message(bytes.as_ref()),
                Message::Text(text) => return Ok(serde_json::from_str(text.as_str())?),
                Message::Ping(bytes) => self
                    .ws
                    .send(Message::Pong(bytes))
                    .await
                    .map_err(|err| Error::Server(err.to_string()))?,
                Message::Close(_) => {
                    return Err(Error::Server("blaze websocket closed".to_string()));
                }
                Message::Pong(_) | Message::Frame(_) => {}
            }
        }

        Err(Error::Server("blaze websocket ended".to_string()))
    }

    pub async fn next_message(&mut self) -> Result<MessageView, Error> {
        loop {
            let envelope = self.next_blaze_message().await?;
            if let Some(error) = envelope.error.clone() {
                return Err(Error::Api(error));
            }
            if let Some(message) = envelope.message_view()? {
                return Ok(message);
            }
        }
    }

    pub async fn close(mut self) -> Result<(), Error> {
        self.ws
            .close(None)
            .await
            .map_err(|err| Error::Server(err.to_string()))
    }

    pub async fn loop_once<L>(
        &mut self,
        safe_user: &SafeUser,
        listener: &mut L,
        options: &BlazeLoopOptions,
    ) -> Result<(), Error>
    where
        L: BlazeListener + Send,
    {
        run_connected_blaze_loop(self, safe_user, listener, options)
            .await
            .map_err(|exit| exit.into_error())
    }
}

pub async fn run_blaze_loop<L>(
    safe_user: &SafeUser,
    listener: &mut L,
    options: BlazeLoopOptions,
) -> Result<(), Error>
where
    L: BlazeListener + Send,
{
    let mut reconnects = 0usize;

    loop {
        let mut client = match BlazeClient::connect(safe_user).await {
            Ok(client) => client,
            Err(Error::Auth(error)) => return Err(Error::Auth(error)),
            Err(error) if options.reconnect => {
                listener
                    .on_disconnect(&error.to_string(), reconnects + 1)
                    .await?;
                reconnects += 1;
                if reconnect_limit_reached(reconnects, options.max_reconnects) {
                    return Err(error);
                }
                tokio::time::sleep(options.reconnect_delay).await;
                continue;
            }
            Err(error) => return Err(error),
        };

        match run_connected_blaze_loop(&mut client, safe_user, listener, &options).await {
            Ok(()) => return Ok(()),
            Err(BlazeLoopExit::Handler(error)) => return Err(error),
            Err(BlazeLoopExit::Connection(error)) if options.reconnect => {
                listener
                    .on_disconnect(&error.to_string(), reconnects + 1)
                    .await?;
                reconnects += 1;
                if reconnect_limit_reached(reconnects, options.max_reconnects) {
                    return Err(error);
                }
                tokio::time::sleep(options.reconnect_delay).await;
            }
            Err(exit) => return Err(exit.into_error()),
        }
    }
}

async fn run_connected_blaze_loop<L>(
    client: &mut BlazeClient,
    safe_user: &SafeUser,
    listener: &mut L,
    options: &BlazeLoopOptions,
) -> Result<(), BlazeLoopExit>
where
    L: BlazeListener + Send,
{
    client
        .list_pending_messages()
        .await
        .map_err(BlazeLoopExit::Connection)?;

    loop {
        let message = client
            .next_message()
            .await
            .map_err(BlazeLoopExit::Connection)?;
        let should_ack = options.sync_ack && listener.sync_ack() && message.should_sync_ack();
        let message_id = message.message_id.clone();

        dispatch_blaze_message(client, listener, message, &safe_user.user_id)
            .await
            .map_err(BlazeLoopExit::Handler)?;

        if should_ack {
            client
                .mark_message_read(&message_id)
                .await
                .map_err(BlazeLoopExit::Connection)?;
        }
    }
}

async fn dispatch_blaze_message<L>(
    client: &mut BlazeClient,
    listener: &mut L,
    message: MessageView,
    client_user_id: &str,
) -> Result<(), Error>
where
    L: BlazeListener + Send,
{
    match message.kind() {
        BlazeMessageKind::AckReceipt => {
            listener
                .on_ack_receipt(client, message, client_user_id)
                .await
        }
        BlazeMessageKind::Transfer => listener.on_transfer(client, message, client_user_id).await,
        BlazeMessageKind::Conversation => {
            listener
                .on_conversation(client, message, client_user_id)
                .await
        }
        BlazeMessageKind::SafeSnapshot => {
            listener
                .on_safe_snapshot(client, message, client_user_id)
                .await
        }
        BlazeMessageKind::SafeInscription => {
            listener
                .on_safe_inscription(client, message, client_user_id)
                .await
        }
        BlazeMessageKind::Message => listener.on_message(client, message, client_user_id).await,
    }
}

fn reconnect_limit_reached(reconnects: usize, max_reconnects: Option<usize>) -> bool {
    max_reconnects
        .map(|max_reconnects| reconnects > max_reconnects)
        .unwrap_or(false)
}

enum BlazeLoopExit {
    Connection(Error),
    Handler(Error),
}

impl BlazeLoopExit {
    fn into_error(self) -> Error {
        match self {
            Self::Connection(error) | Self::Handler(error) => error,
        }
    }
}

fn ensure_rustls_provider() {
    RUSTLS_PROVIDER.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

pub async fn connect_blaze(safe_user: &SafeUser) -> Result<BlazeClient, Error> {
    BlazeClient::connect(safe_user).await
}

pub fn encode_blaze_message(message: &BlazeMessage) -> Result<Vec<u8>, Error> {
    let json = serde_json::to_vec(message)?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::new(3));
    encoder.write_all(&json)?;
    Ok(encoder.finish()?)
}

pub fn decode_blaze_message(data: &[u8]) -> Result<BlazeMessage, Error> {
    let mut decoder = GzDecoder::new(data);
    let mut json = Vec::new();
    decoder.read_to_end(&mut json)?;
    Ok(serde_json::from_slice(&json)?)
}

#[cfg(test)]
mod tests {
    use crate::message::{
        MESSAGE_CATEGORY_PLAIN_TEXT, MESSAGE_CATEGORY_SYSTEM_ACCOUNT_SNAPSHOT,
        MESSAGE_CATEGORY_SYSTEM_CONVERSATION, MESSAGE_CATEGORY_SYSTEM_SAFE_INSCRIPTION,
        MESSAGE_CATEGORY_SYSTEM_SAFE_SNAPSHOT, encode_message_data,
    };

    use super::*;

    #[test]
    fn test_blaze_message_gzip_roundtrip() {
        let message = BlazeMessage::new(
            BLAZE_ACTION_ACKNOWLEDGE_MESSAGE_RECEIPT,
            Some(json!({"message_id": "message-id", "status": "READ"})),
        );

        let encoded = encode_blaze_message(&message).unwrap();
        let decoded = decode_blaze_message(&encoded).unwrap();
        assert_eq!(decoded.id, message.id);
        assert_eq!(decoded.action, BLAZE_ACTION_ACKNOWLEDGE_MESSAGE_RECEIPT);
        assert_eq!(decoded.params.unwrap()["message_id"], "message-id");
    }

    #[test]
    fn test_blaze_message_view_decode() {
        let envelope = BlazeMessage {
            id: "envelope-id".to_string(),
            action: BLAZE_ACTION_CREATE_MESSAGE.to_string(),
            params: None,
            data: Some(json!({
                "type": "message",
                "conversation_id": "conversation-id",
                "user_id": "user-id",
                "message_id": "message-id",
                "category": MESSAGE_CATEGORY_PLAIN_TEXT,
                "data_base64": encode_message_data("hello"),
                "status": "SENT",
                "source": "CREATE_MESSAGE"
            })),
            error: None,
        };

        let message = envelope.message_view().unwrap().unwrap();
        assert_eq!(message.conversation_id, "conversation-id");
        assert_eq!(message.category, MESSAGE_CATEGORY_PLAIN_TEXT);
        assert_eq!(message.data_text().unwrap(), "hello");
    }

    #[test]
    fn test_blaze_message_view_fills_missing_source_from_envelope_action() {
        let envelope = BlazeMessage {
            id: "envelope-id".to_string(),
            action: BLAZE_ACTION_ACKNOWLEDGE_MESSAGE_RECEIPT.to_string(),
            params: None,
            data: Some(json!({
                "conversation_id": "conversation-id",
                "user_id": "user-id",
                "message_id": "message-id",
                "category": MESSAGE_CATEGORY_PLAIN_TEXT,
                "data_base64": encode_message_data("hello")
            })),
            error: None,
        };

        let message = envelope.message_view().unwrap().unwrap();
        assert_eq!(
            message.source.as_deref(),
            Some(BLAZE_ACTION_ACKNOWLEDGE_MESSAGE_RECEIPT)
        );
        assert_eq!(message.kind(), BlazeMessageKind::AckReceipt);
        assert!(!message.should_sync_ack());
    }

    #[test]
    fn test_blaze_message_kind_routes_special_categories() {
        let mut message = MessageView {
            message_type: Some("message".to_string()),
            representative_id: None,
            quote_message_id: None,
            conversation_id: "conversation-id".to_string(),
            user_id: "user-id".to_string(),
            session_id: None,
            message_id: "message-id".to_string(),
            category: MESSAGE_CATEGORY_PLAIN_TEXT.to_string(),
            data: None,
            data_base64: encode_message_data("hello"),
            status: None,
            source: None,
            created_at: None,
            updated_at: None,
        };

        assert_eq!(message.kind(), BlazeMessageKind::Message);
        assert!(message.should_sync_ack());

        message.category = MESSAGE_CATEGORY_SYSTEM_ACCOUNT_SNAPSHOT.to_string();
        assert_eq!(message.kind(), BlazeMessageKind::Transfer);

        message.category = MESSAGE_CATEGORY_SYSTEM_CONVERSATION.to_string();
        assert_eq!(message.kind(), BlazeMessageKind::Conversation);

        message.category = MESSAGE_CATEGORY_SYSTEM_SAFE_SNAPSHOT.to_string();
        assert_eq!(message.kind(), BlazeMessageKind::SafeSnapshot);

        message.category = MESSAGE_CATEGORY_SYSTEM_SAFE_INSCRIPTION.to_string();
        assert_eq!(message.kind(), BlazeMessageKind::SafeInscription);

        message.source = Some(BLAZE_ACTION_ACKNOWLEDGE_MESSAGE_RECEIPT.to_string());
        assert_eq!(message.kind(), BlazeMessageKind::AckReceipt);
    }

    #[test]
    fn test_blaze_loop_options_default() {
        let options = BlazeLoopOptions::default();
        assert!(options.sync_ack);
        assert!(options.reconnect);
        assert_eq!(options.reconnect_delay, std::time::Duration::from_secs(5));
        assert_eq!(options.max_reconnects, None);
    }

    #[test]
    fn test_reconnect_limit() {
        assert!(reconnect_limit_reached(1, Some(0)));
        assert!(!reconnect_limit_reached(1, Some(1)));
        assert!(reconnect_limit_reached(2, Some(1)));
        assert!(!reconnect_limit_reached(100, None));
    }

    #[test]
    fn test_blaze_non_message_action_returns_none() {
        let envelope = BlazeMessage::new(BLAZE_ACTION_LIST_PENDING_MESSAGES, None);
        assert!(envelope.message_view().unwrap().is_none());
    }
}
