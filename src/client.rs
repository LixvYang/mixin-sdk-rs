use std::path::Path;

use serde::Serialize;

use crate::{
    address::{self, Address, AddressInput, SimpleAddress},
    app::{
        self, App, AppBilling, AppProperty, AppRegistration, AppRequest,
        AppSafeRegistrationRequest, AppSafeSessionRequest, AppSecret, AppSession,
    },
    asset::{self, AssetFee, AssetNetwork},
    attachment::{self, Attachment, UploadedAttachment},
    blaze::{self, BlazeClient},
    chain::{self, NetworkChain},
    circle::{self, Circle, CircleConversation, CircleConversationQuery},
    code::{self, Scheme},
    collectible::{
        self, CollectibleCollection, CollectibleOutputQuery, CollectibleOutputsRequest,
        CollectibleTransaction,
    },
    computer::{
        self, ComputerDeployedAsset, ComputerFee, ComputerInfo, ComputerNonceAccount,
        ComputerRegisterPreview, ComputerSystemCallResponse, ComputerUser,
    },
    conversation::{self, Conversation, Participant},
    deposit::{
        self, DepositEntry, DepositEntryRequest, SafePendingDeposit, SafePendingDepositQuery,
    },
    error::Error,
    external::{self, ExternalProxyRequest, ExternalTransaction, ExternalTransactionQuery},
    fiats::{self, Fiat},
    inscription::{
        self, Collection, Inscription, InscriptionDeploy, InscriptionDistribute,
        InscriptionInscribe, InscriptionOccupy,
    },
    legacy_multisig::{self, LegacyMultisigQuery, LegacyMultisigRequest, LegacyMultisigUtxo},
    legacy_transfer::{
        self, LegacyGhostInputRequest, LegacyGhostKeys, LegacyRawTransactionRequest,
        LegacySnapshot, LegacyTransferRequest,
    },
    message::{
        self, AppButtonPayload, AppCardMessagePayload, AudioMessagePayload, ContactMessagePayload,
        FileMessagePayload, ImageMessagePayload, LiveMessagePayload, LocationMessagePayload,
        MessageRequest, MessageSession, RecallMessagePayload, ReceiptAcknowledgementRequest,
        StickerMessagePayload, TransferMessagePayload, VideoMessagePayload,
    },
    models::{Asset, CollectibleOutput, CollectibleToken, Output, Snapshot},
    network::{self, AssetTicker, NetworkInfo, NetworkSnapshot, NetworkSnapshotQuery},
    oauth::{self, AccessTokenRequest, AccessTokenResponse, Authorization, AuthorizeRequest},
    output,
    payment::{self, Payment, PaymentRequest, RawPaymentRequest, TransferPaymentRequest},
    pin,
    safe::{self, GhostKeyRequest, GhostKeys, SafeUser},
    safe_multisig::{self, SafeMultisigRequest},
    session::{self, UserSession},
    snapshot::{
        self, MessageWithSession, SafeSnapshot, SafeSnapshotNotificationRequest, SafeSnapshotQuery,
        SnapshotQuery,
    },
    transaction::{self, TransactionRequest, TransactionView},
    user::{self, Log, LogQuery, PreferenceRequest, RelationshipRequest, User},
    withdrawal::{self, WithdrawalView},
};

/// A convenience facade around a `SafeUser`.
///
/// The lower-level module functions remain available for callers who prefer the
/// Go-style `function(..., safe_user)` shape. `Client` gives application code a
/// single object for common authenticated Bot and Safe API calls.
#[derive(Debug, Clone)]
pub struct Client {
    safe_user: SafeUser,
}

impl Client {
    pub fn new(safe_user: SafeUser) -> Self {
        Self { safe_user }
    }

    pub fn from_keystore_file(path: &str) -> Result<Self, Error> {
        Ok(Self::new(SafeUser::new_from_file(path)?))
    }

    pub fn from_env() -> Result<Self, Error> {
        Ok(Self::new(SafeUser::new_from_env()?))
    }

    pub fn from_env_str(env: &str) -> Result<Self, Error> {
        Ok(Self::new(SafeUser::new_from_env_str(env)?))
    }

    pub fn safe_user(&self) -> &SafeUser {
        &self.safe_user
    }

    pub fn into_safe_user(self) -> SafeUser {
        self.safe_user
    }

    pub fn user_id(&self) -> &str {
        &self.safe_user.user_id
    }

    pub fn session_id(&self) -> &str {
        &self.safe_user.session_id
    }

    pub async fn me(&self) -> Result<User, Error> {
        user::request_user_me(&self.safe_user).await
    }

    pub async fn get_user(&self, user_id: &str) -> Result<User, Error> {
        user::get_user(&self.safe_user, user_id).await
    }

    pub async fn search_user(&self, query: &str) -> Result<Vec<User>, Error> {
        user::search_user(query, &self.safe_user).await
    }

    pub async fn search_user_one(&self, query: &str) -> Result<User, Error> {
        user::search_user_one(query, &self.safe_user).await
    }

    pub async fn get_users(&self, user_ids: &[String]) -> Result<Vec<User>, Error> {
        user::get_users(&self.safe_user, user_ids).await
    }

    pub async fn get_friends(&self) -> Result<Vec<User>, Error> {
        user::get_friends(&self.safe_user).await
    }

    pub async fn get_blocking_users(&self) -> Result<Vec<User>, Error> {
        user::get_blocking_users(&self.safe_user).await
    }

    pub async fn rotate_code(&self) -> Result<User, Error> {
        user::rotate_code(&self.safe_user).await
    }

    pub async fn update_user_me(
        &self,
        full_name: &str,
        avatar_base64: &str,
    ) -> Result<User, Error> {
        user::update_user_me(full_name, avatar_base64, &self.safe_user).await
    }

    pub async fn update_preferences(&self, request: &PreferenceRequest) -> Result<User, Error> {
        user::update_preferences(request, &self.safe_user).await
    }

    pub async fn update_preference(
        &self,
        message_source: &str,
        conversation_source: &str,
        currency: &str,
        threshold: &f64,
        confirmation_threshold: Option<&f64>,
    ) -> Result<User, Error> {
        user::update_preference(
            message_source,
            conversation_source,
            currency,
            threshold,
            confirmation_threshold,
            &self.safe_user,
        )
        .await
    }

    pub async fn relationship(&self, user_id: &str, action: &str) -> Result<User, Error> {
        user::relationship(user_id, action, &self.safe_user).await
    }

    pub async fn update_relationship(&self, request: &RelationshipRequest) -> Result<User, Error> {
        user::update_relationship(request, &self.safe_user).await
    }

    pub async fn list_logs(&self, query: &LogQuery) -> Result<Vec<Log>, Error> {
        user::list_logs(query, &self.safe_user).await
    }

    pub async fn get_app(&self, app_id: &str) -> Result<App, Error> {
        app::get_app(app_id, &self.safe_user).await
    }

    pub async fn current_app(&self) -> Result<App, Error> {
        self.get_app(&self.safe_user.user_id).await
    }

    pub async fn app_creator_id(&self) -> Result<String, Error> {
        self.current_app()
            .await?
            .creator_id
            .ok_or_else(|| Error::DataNotFound("app response is missing creator_id".to_string()))
    }

    pub async fn list_apps(&self) -> Result<Vec<App>, Error> {
        app::list_apps(&self.safe_user).await
    }

    pub async fn get_app_property(&self) -> Result<AppProperty, Error> {
        app::get_app_property(&self.safe_user).await
    }

    pub async fn get_app_billing(&self, app_id: &str) -> Result<AppBilling, Error> {
        app::get_app_billing(app_id, &self.safe_user).await
    }

    pub async fn list_favorite_apps(&self, user_id: &str) -> Result<Vec<App>, Error> {
        app::list_favorite_apps(user_id, &self.safe_user).await
    }

    pub async fn create_app(&self, request: &AppRequest) -> Result<App, Error> {
        app::create_app(request, &self.safe_user).await
    }

    pub async fn update_app(&self, app_id: &str, request: &AppRequest) -> Result<App, Error> {
        app::update_app(app_id, request, &self.safe_user).await
    }

    pub async fn update_app_secret(&self, app_id: &str) -> Result<AppSecret, Error> {
        app::update_app_secret(app_id, &self.safe_user).await
    }

    pub async fn update_safe_app_session(
        &self,
        app_id: &str,
        request: &AppSafeSessionRequest,
    ) -> Result<AppSession, Error> {
        app::update_safe_app_session(app_id, request, &self.safe_user).await
    }

    pub async fn register_safe_app(
        &self,
        app_id: &str,
        request: &AppSafeRegistrationRequest,
    ) -> Result<AppRegistration, Error> {
        app::register_safe_app(app_id, request, &self.safe_user).await
    }

    pub async fn favorite_app(&self, app_id: &str) -> Result<Vec<App>, Error> {
        app::favorite_app(app_id, &self.safe_user).await
    }

    pub async fn unfavorite_app(&self, app_id: &str) -> Result<(), Error> {
        app::unfavorite_app(app_id, &self.safe_user).await
    }

    pub async fn migrate_app(&self, receiver_id: &str) -> Result<App, Error> {
        app::migrate_app(receiver_id, &self.safe_user).await
    }

    pub async fn fetch_user_sessions(
        &self,
        user_ids: &[String],
    ) -> Result<Vec<UserSession>, Error> {
        session::fetch_user_sessions(user_ids, &self.safe_user).await
    }

    pub async fn fetch_user_session(&self, user_id: &str) -> Result<Option<UserSession>, Error> {
        session::fetch_user_session(user_id, &self.safe_user).await
    }

    pub async fn send_message_request(
        &self,
        message: MessageRequest,
    ) -> Result<MessageRequest, Error> {
        message::send_message_request(message, &self.safe_user).await
    }

    pub async fn post_message(&self, message: MessageRequest) -> Result<(), Error> {
        message::post_message(message, &self.safe_user).await
    }

    pub async fn post_messages(&self, messages: &[MessageRequest]) -> Result<(), Error> {
        message::post_messages(messages, &self.safe_user).await
    }

    pub async fn send_data_message(
        &self,
        recipient_id: &str,
        category: &str,
        data: impl AsRef<[u8]>,
    ) -> Result<MessageRequest, Error> {
        message::send_data_message(recipient_id, category, data, &self.safe_user).await
    }

    pub async fn send_json_message<T>(
        &self,
        recipient_id: &str,
        category: &str,
        payload: &T,
    ) -> Result<MessageRequest, Error>
    where
        T: Serialize + ?Sized,
    {
        message::send_json_message(recipient_id, category, payload, &self.safe_user).await
    }

    pub async fn send_text_message(
        &self,
        recipient_id: &str,
        text: &str,
    ) -> Result<MessageRequest, Error> {
        message::send_text_message(recipient_id, text, &self.safe_user).await
    }

    pub async fn send_post_message(
        &self,
        recipient_id: &str,
        text: &str,
    ) -> Result<MessageRequest, Error> {
        message::send_post_message(recipient_id, text, &self.safe_user).await
    }

    pub async fn send_encrypted_text_message(
        &self,
        recipient_id: &str,
        text: &str,
        sessions: &[MessageSession],
    ) -> Result<MessageRequest, Error> {
        message::send_encrypted_text_message(recipient_id, text, sessions, &self.safe_user).await
    }

    pub async fn send_encrypted_text_message_to_user(
        &self,
        recipient_id: &str,
        text: &str,
    ) -> Result<MessageRequest, Error> {
        message::send_encrypted_text_message_to_user(recipient_id, text, &self.safe_user).await
    }

    pub async fn send_sticker_message(
        &self,
        recipient_id: &str,
        sticker: &StickerMessagePayload,
    ) -> Result<MessageRequest, Error> {
        message::send_sticker_message(recipient_id, sticker, &self.safe_user).await
    }

    pub async fn send_image_message(
        &self,
        recipient_id: &str,
        image: &ImageMessagePayload,
    ) -> Result<MessageRequest, Error> {
        message::send_image_message(recipient_id, image, &self.safe_user).await
    }

    pub async fn send_audio_message(
        &self,
        recipient_id: &str,
        audio: &AudioMessagePayload,
    ) -> Result<MessageRequest, Error> {
        message::send_audio_message(recipient_id, audio, &self.safe_user).await
    }

    pub async fn send_video_message(
        &self,
        recipient_id: &str,
        video: &VideoMessagePayload,
    ) -> Result<MessageRequest, Error> {
        message::send_video_message(recipient_id, video, &self.safe_user).await
    }

    pub async fn send_contact_message(
        &self,
        recipient_id: &str,
        contact: &ContactMessagePayload,
    ) -> Result<MessageRequest, Error> {
        message::send_contact_message(recipient_id, contact, &self.safe_user).await
    }

    pub async fn send_app_card_message(
        &self,
        recipient_id: &str,
        app_card: &AppCardMessagePayload,
    ) -> Result<MessageRequest, Error> {
        message::send_app_card_message(recipient_id, app_card, &self.safe_user).await
    }

    pub async fn send_file_message(
        &self,
        recipient_id: &str,
        file: &FileMessagePayload,
    ) -> Result<MessageRequest, Error> {
        message::send_file_message(recipient_id, file, &self.safe_user).await
    }

    pub async fn send_live_message(
        &self,
        recipient_id: &str,
        live: &LiveMessagePayload,
    ) -> Result<MessageRequest, Error> {
        message::send_live_message(recipient_id, live, &self.safe_user).await
    }

    pub async fn send_location_message(
        &self,
        recipient_id: &str,
        location: &LocationMessagePayload,
    ) -> Result<MessageRequest, Error> {
        message::send_location_message(recipient_id, location, &self.safe_user).await
    }

    pub async fn send_app_button_group_message(
        &self,
        recipient_id: &str,
        buttons: &[AppButtonPayload],
    ) -> Result<MessageRequest, Error> {
        message::send_app_button_group_message(recipient_id, buttons, &self.safe_user).await
    }

    pub async fn send_transfer_message(
        &self,
        recipient_id: &str,
        transfer: &TransferMessagePayload,
    ) -> Result<MessageRequest, Error> {
        message::send_transfer_message(recipient_id, transfer, &self.safe_user).await
    }

    pub async fn send_recall_message(
        &self,
        recipient_id: &str,
        recall: &RecallMessagePayload,
    ) -> Result<MessageRequest, Error> {
        message::send_recall_message(recipient_id, recall, &self.safe_user).await
    }

    pub async fn post_acknowledgement(
        &self,
        request: ReceiptAcknowledgementRequest,
    ) -> Result<(), Error> {
        message::post_acknowledgement(request, &self.safe_user).await
    }

    pub async fn post_acknowledgements(
        &self,
        requests: &[ReceiptAcknowledgementRequest],
    ) -> Result<(), Error> {
        message::post_acknowledgements(requests, &self.safe_user).await
    }

    pub async fn acknowledge_message(&self, message_id: &str, status: &str) -> Result<(), Error> {
        message::acknowledge_message(message_id, status, &self.safe_user).await
    }

    pub async fn mark_message_read(&self, message_id: &str) -> Result<(), Error> {
        message::mark_message_read(message_id, &self.safe_user).await
    }

    pub async fn list_assets(&self) -> Result<Vec<Asset>, Error> {
        asset::list_assets(&self.safe_user).await
    }

    pub async fn read_asset(&self, asset_id: &str) -> Result<Asset, Error> {
        asset::read_asset(asset_id, &self.safe_user).await
    }

    pub async fn fetch_assets(&self, asset_ids: &[String]) -> Result<Vec<Asset>, Error> {
        asset::fetch_assets(asset_ids, &self.safe_user).await
    }

    pub async fn read_asset_fees(
        &self,
        asset_id: &str,
        destination: &str,
    ) -> Result<Vec<AssetFee>, Error> {
        asset::read_asset_fees(asset_id, destination, &self.safe_user).await
    }

    pub async fn read_code_value(&self, code_id: &str) -> Result<serde_json::Value, Error> {
        code::read_code_value(code_id).await
    }

    pub async fn read_code<T>(&self, code_id: &str) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        code::read_code(code_id).await
    }

    pub async fn create_scheme(&self, target: &str) -> Result<Scheme, Error> {
        code::create_scheme(target, &self.safe_user).await
    }

    pub async fn get_oauth_token(
        &self,
        request: &AccessTokenRequest,
    ) -> Result<AccessTokenResponse, Error> {
        oauth::get_token(request).await
    }

    pub async fn authorize_oauth(
        &self,
        request: &AuthorizeRequest,
    ) -> Result<Authorization, Error> {
        oauth::authorize(request, &self.safe_user).await
    }

    pub async fn list_authorizations(
        &self,
        app_id: Option<&str>,
    ) -> Result<Vec<Authorization>, Error> {
        oauth::list_authorizations(app_id, &self.safe_user).await
    }

    pub async fn revoke_authorize(&self, client_id: &str) -> Result<(), Error> {
        oauth::revoke_authorize(client_id, &self.safe_user).await
    }

    pub async fn create_payment(&self, request: &PaymentRequest) -> Result<Payment, Error> {
        payment::create_payment(request, &self.safe_user).await
    }

    pub async fn create_transfer_payment(
        &self,
        request: &TransferPaymentRequest,
    ) -> Result<Payment, Error> {
        payment::create_transfer_payment(request, &self.safe_user).await
    }

    pub async fn create_raw_payment(&self, request: &RawPaymentRequest) -> Result<Payment, Error> {
        payment::create_raw_payment(request, &self.safe_user).await
    }

    pub async fn get_fiats(&self) -> Result<Vec<Fiat>, Error> {
        fiats::get_fiats().await
    }

    pub async fn read_network_info(&self) -> Result<NetworkInfo, Error> {
        network::read_network_info().await
    }

    pub async fn read_network_chain(&self, chain_id: &str) -> Result<NetworkChain, Error> {
        chain::read_network_chain(chain_id).await
    }

    pub async fn read_network_chains(&self) -> Result<Vec<NetworkChain>, Error> {
        chain::read_network_chains().await
    }

    pub async fn read_network_assets(&self) -> Result<Vec<AssetNetwork>, Error> {
        asset::read_network_assets().await
    }

    pub async fn read_network_assets_top(&self) -> Result<Vec<AssetNetwork>, Error> {
        asset::read_network_assets_top().await
    }

    pub async fn read_network_asset(&self, asset_id: &str) -> Result<AssetNetwork, Error> {
        asset::read_network_asset(asset_id).await
    }

    pub async fn search_network_assets(
        &self,
        keyword: &str,
        kind: Option<&str>,
    ) -> Result<Vec<AssetNetwork>, Error> {
        network::search_network_assets(keyword, kind).await
    }

    pub async fn read_network_assets_top_with_kind(
        &self,
        kind: Option<&str>,
    ) -> Result<Vec<AssetNetwork>, Error> {
        network::read_network_assets_top(kind).await
    }

    pub async fn read_asset_ticker(
        &self,
        asset_id: &str,
        offset: Option<&str>,
    ) -> Result<AssetTicker, Error> {
        network::read_asset_ticker(asset_id, offset).await
    }

    pub async fn read_network_snapshot(&self, snapshot_id: &str) -> Result<NetworkSnapshot, Error> {
        network::read_network_snapshot(snapshot_id).await
    }

    pub async fn list_network_snapshots(
        &self,
        query: &NetworkSnapshotQuery,
    ) -> Result<Vec<NetworkSnapshot>, Error> {
        network::list_network_snapshots(query).await
    }

    pub async fn external_transactions(
        &self,
        query: &ExternalTransactionQuery,
    ) -> Result<Vec<ExternalTransaction>, Error> {
        external::external_transactions(query).await
    }

    pub async fn external_proxy(
        &self,
        request: &ExternalProxyRequest,
    ) -> Result<serde_json::Value, Error> {
        external::external_proxy(request).await
    }

    pub async fn create_deposit_entries(
        &self,
        request: &DepositEntryRequest,
    ) -> Result<Vec<DepositEntry>, Error> {
        deposit::create_deposit_entries(request, &self.safe_user).await
    }

    pub async fn create_deposit_entry(
        &self,
        chain_id: &str,
        members: &[String],
        threshold: i64,
    ) -> Result<Vec<DepositEntry>, Error> {
        deposit::create_deposit_entry(chain_id, members, threshold, &self.safe_user).await
    }

    pub async fn create_primary_deposit_entry(
        &self,
        chain_id: &str,
    ) -> Result<Vec<DepositEntry>, Error> {
        deposit::create_primary_deposit_entry(chain_id, &self.safe_user).await
    }

    pub async fn fetch_pending_safe_deposits(
        &self,
        query: &SafePendingDepositQuery,
    ) -> Result<Vec<SafePendingDeposit>, Error> {
        deposit::fetch_pending_safe_deposits(query).await
    }

    pub async fn list_outputs(
        &self,
        members_hash: &str,
        threshold: u8,
        asset_id: Option<&str>,
        state: Option<&str>,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<Output>, Error> {
        output::list_outputs(
            members_hash,
            threshold,
            asset_id,
            state,
            offset,
            limit,
            &self.safe_user,
        )
        .await
    }

    pub async fn list_unspent_outputs(
        &self,
        members_hash: &str,
        threshold: u8,
        asset_id: Option<&str>,
    ) -> Result<Vec<Output>, Error> {
        output::list_unspent_outputs(members_hash, threshold, asset_id, &self.safe_user).await
    }

    pub async fn get_output(&self, output_id: &str) -> Result<Output, Error> {
        output::get_output(output_id, &self.safe_user).await
    }

    pub async fn fetch_safe_outputs(&self, output_ids: &[String]) -> Result<Vec<Output>, Error> {
        output::fetch_safe_outputs(&self.safe_user.user_id, output_ids, &self.safe_user).await
    }

    pub async fn fetch_safe_outputs_for_user(
        &self,
        user_id: &str,
        output_ids: &[String],
    ) -> Result<Vec<Output>, Error> {
        output::fetch_safe_outputs(user_id, output_ids, &self.safe_user).await
    }

    pub async fn request_safe_ghost_keys(
        &self,
        requests: &[GhostKeyRequest],
    ) -> Result<Vec<GhostKeys>, Error> {
        safe::request_safe_ghost_keys(requests, &self.safe_user).await
    }

    pub async fn register_safe_user(&self) -> Result<User, Error> {
        safe::register_safe_user(&self.safe_user).await
    }

    pub async fn verify_tip(&self) -> Result<User, Error> {
        safe::verify_tip(&self.safe_user).await
    }

    pub async fn verify_pin(&self, pin_hex: &str) -> Result<User, Error> {
        pin::verify_pin(pin_hex, &self.safe_user).await
    }

    pub async fn verify_pin_with_encrypted(&self, pin_base64: &str) -> Result<User, Error> {
        pin::verify_pin_with_encrypted(pin_base64, &self.safe_user).await
    }

    pub async fn update_pin(
        &self,
        old_pin_hex: &str,
        new_pin_hex: &str,
    ) -> Result<Option<User>, Error> {
        pin::update_pin(old_pin_hex, new_pin_hex, &self.safe_user).await
    }

    pub async fn update_tip_pin(
        &self,
        old_pin_hex: &str,
        public_tip_hex: &str,
        counter: u64,
    ) -> Result<Option<User>, Error> {
        pin::update_tip_pin(old_pin_hex, public_tip_hex, counter, &self.safe_user).await
    }

    pub async fn update_pin_with_encrypted(
        &self,
        old_pin_base64: &str,
        pin_base64: &str,
    ) -> Result<Option<User>, Error> {
        pin::update_pin_with_encrypted(old_pin_base64, pin_base64, &self.safe_user).await
    }

    pub async fn create_attachment(&self) -> Result<Attachment, Error> {
        attachment::create_attachment(&self.safe_user).await
    }

    pub async fn fetch_attachment(&self, attachment_id: &str) -> Result<Attachment, Error> {
        attachment::fetch_attachment(attachment_id, &self.safe_user).await
    }

    pub async fn upload_attachment_bytes(
        &self,
        bytes: impl Into<Vec<u8>>,
    ) -> Result<UploadedAttachment, Error> {
        attachment::upload_attachment_bytes(bytes, &self.safe_user).await
    }

    pub async fn upload_attachment_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<UploadedAttachment, Error> {
        attachment::upload_attachment_file(path, &self.safe_user).await
    }

    pub async fn create_address(&self, input: &AddressInput<'_>) -> Result<Address, Error> {
        address::create_address(input, &self.safe_user).await
    }

    pub async fn read_address(&self, address_id: &str) -> Result<Address, Error> {
        address::read_address(address_id, &self.safe_user).await
    }

    pub async fn delete_address(&self, address_id: &str) -> Result<(), Error> {
        address::delete_address(address_id, &self.safe_user).await
    }

    pub async fn list_addresses_by_asset(&self, asset_id: &str) -> Result<Vec<Address>, Error> {
        address::list_addresses_by_asset(asset_id, &self.safe_user).await
    }

    pub async fn check_address(
        &self,
        asset_id: &str,
        destination: &str,
        tag: Option<&str>,
    ) -> Result<SimpleAddress, Error> {
        address::check_address(asset_id, destination, tag).await
    }

    pub async fn create_withdrawal(
        &self,
        address_id: &str,
        amount: &str,
        fee: &str,
        trace_id: &str,
        memo: Option<&str>,
    ) -> Result<WithdrawalView, Error> {
        withdrawal::create_withdrawal(address_id, amount, fee, trace_id, memo, &self.safe_user)
            .await
    }

    pub async fn list_snapshots(&self, query: &SnapshotQuery) -> Result<Vec<Snapshot>, Error> {
        snapshot::list_snapshots(query, &self.safe_user).await
    }

    pub async fn read_snapshot(&self, snapshot_id: &str) -> Result<Snapshot, Error> {
        snapshot::read_snapshot(snapshot_id, &self.safe_user).await
    }

    pub async fn list_safe_snapshots(
        &self,
        query: &SafeSnapshotQuery,
    ) -> Result<Vec<SafeSnapshot>, Error> {
        snapshot::list_safe_snapshots(query, &self.safe_user).await
    }

    pub async fn read_safe_snapshot(&self, snapshot_id: &str) -> Result<SafeSnapshot, Error> {
        snapshot::read_safe_snapshot(snapshot_id, &self.safe_user).await
    }

    pub async fn notify_safe_snapshot(
        &self,
        request: &SafeSnapshotNotificationRequest,
    ) -> Result<MessageWithSession, Error> {
        snapshot::notify_safe_snapshot(request, &self.safe_user).await
    }

    pub async fn fetch_transfer_by_trace(&self, trace_id: &str) -> Result<LegacySnapshot, Error> {
        legacy_transfer::fetch_transfer_by_trace(trace_id, &self.safe_user).await
    }

    pub async fn create_legacy_transfer(
        &self,
        request: &LegacyTransferRequest,
    ) -> Result<LegacySnapshot, Error> {
        legacy_transfer::create_transfer(request, &self.safe_user).await
    }

    pub async fn create_legacy_transfer_with_pin(
        &self,
        pin_hex: &str,
        request: &LegacyTransferRequest,
    ) -> Result<LegacySnapshot, Error> {
        legacy_transfer::create_transfer_with_pin(pin_hex, request, &self.safe_user).await
    }

    pub async fn send_legacy_raw_transaction(
        &self,
        request: &LegacyRawTransactionRequest,
    ) -> Result<LegacySnapshot, Error> {
        legacy_transfer::send_raw_transaction(request, &self.safe_user).await
    }

    pub async fn send_legacy_raw_transaction_with_pin(
        &self,
        pin_hex: &str,
        request: &LegacyRawTransactionRequest,
    ) -> Result<LegacySnapshot, Error> {
        legacy_transfer::send_raw_transaction_with_pin(pin_hex, request, &self.safe_user).await
    }

    pub async fn request_legacy_ghost_keys(
        &self,
        requests: &[LegacyGhostInputRequest],
    ) -> Result<Vec<LegacyGhostKeys>, Error> {
        legacy_transfer::request_legacy_ghost_keys(requests, &self.safe_user).await
    }

    pub async fn create_contact_conversation(
        &self,
        participant_id: &str,
    ) -> Result<Conversation, Error> {
        conversation::create_contact_conversation(participant_id, &self.safe_user).await
    }

    pub async fn create_group_conversation(
        &self,
        name: &str,
        announcement: &str,
        participants: Vec<Participant>,
    ) -> Result<Conversation, Error> {
        conversation::create_group_conversation(name, announcement, participants, &self.safe_user)
            .await
    }

    pub async fn get_conversation(&self, conversation_id: &str) -> Result<Conversation, Error> {
        conversation::get_conversation(conversation_id, &self.safe_user).await
    }

    pub async fn join_conversation(&self, conversation_id: &str) -> Result<Conversation, Error> {
        conversation::join_conversation(conversation_id, &self.safe_user).await
    }

    pub async fn rotate_conversation(&self, conversation_id: &str) -> Result<Conversation, Error> {
        conversation::rotate_conversation(conversation_id, &self.safe_user).await
    }

    pub async fn update_participants(
        &self,
        conversation_id: &str,
        action: &str,
        participants: Vec<Participant>,
    ) -> Result<Conversation, Error> {
        conversation::update_participants(conversation_id, action, participants, &self.safe_user)
            .await
    }

    pub async fn mute_conversation(
        &self,
        conversation_id: &str,
        duration: i64,
    ) -> Result<Conversation, Error> {
        conversation::mute_conversation(conversation_id, duration, &self.safe_user).await
    }

    pub async fn get_circle(&self, circle_id: &str) -> Result<Circle, Error> {
        circle::get_circle(circle_id, &self.safe_user).await
    }

    pub async fn list_circles(&self) -> Result<Vec<Circle>, Error> {
        circle::list_circles(&self.safe_user).await
    }

    pub async fn list_circle_conversations(
        &self,
        circle_id: &str,
        query: &CircleConversationQuery,
    ) -> Result<Vec<CircleConversation>, Error> {
        circle::list_circle_conversations(circle_id, query, &self.safe_user).await
    }

    pub async fn create_circle(&self, name: &str) -> Result<Circle, Error> {
        circle::create_circle(name, &self.safe_user).await
    }

    pub async fn update_circle(&self, circle_id: &str, name: &str) -> Result<Circle, Error> {
        circle::update_circle(circle_id, name, &self.safe_user).await
    }

    pub async fn delete_circle(&self, circle_id: &str) -> Result<(), Error> {
        circle::delete_circle(circle_id, &self.safe_user).await
    }

    pub async fn add_user_to_circle(
        &self,
        user_id: &str,
        circle_id: &str,
    ) -> Result<Vec<Circle>, Error> {
        circle::add_user_to_circle(user_id, circle_id, &self.safe_user).await
    }

    pub async fn remove_user_from_circle(
        &self,
        user_id: &str,
        circle_id: &str,
    ) -> Result<Vec<Circle>, Error> {
        circle::remove_user_from_circle(user_id, circle_id, &self.safe_user).await
    }

    pub async fn add_conversation_to_circle(
        &self,
        conversation_id: &str,
        circle_id: &str,
    ) -> Result<Vec<Circle>, Error> {
        circle::add_conversation_to_circle(conversation_id, circle_id, &self.safe_user).await
    }

    pub async fn remove_conversation_from_circle(
        &self,
        conversation_id: &str,
        circle_id: &str,
    ) -> Result<Vec<Circle>, Error> {
        circle::remove_conversation_from_circle(conversation_id, circle_id, &self.safe_user).await
    }

    pub async fn read_collectible_token(&self, token_id: &str) -> Result<CollectibleToken, Error> {
        collectible::read_collectible_token(token_id, &self.safe_user).await
    }

    pub async fn read_collectible_collection(
        &self,
        collection_id: &str,
    ) -> Result<CollectibleCollection, Error> {
        collectible::read_collectible_collection(collection_id, &self.safe_user).await
    }

    pub async fn list_collectible_outputs(
        &self,
        query: &CollectibleOutputQuery,
    ) -> Result<Vec<CollectibleOutput>, Error> {
        collectible::list_collectible_outputs(query, &self.safe_user).await
    }

    pub async fn list_collectible_outputs_for_members(
        &self,
        request: &CollectibleOutputsRequest,
    ) -> Result<Vec<CollectibleOutput>, Error> {
        collectible::list_collectible_outputs_for_members(request, &self.safe_user).await
    }

    pub async fn create_collectible_transfer(
        &self,
        action: &str,
        raw: &str,
    ) -> Result<CollectibleTransaction, Error> {
        collectible::create_collectible_transfer(action, raw, &self.safe_user).await
    }

    pub async fn sign_collectible_request(
        &self,
        request_id: &str,
        pin_base64: &str,
    ) -> Result<CollectibleTransaction, Error> {
        collectible::sign_collectible_request(request_id, pin_base64, &self.safe_user).await
    }

    pub async fn sign_collectible_request_with_pin(
        &self,
        request_id: &str,
        pin_hex: &str,
    ) -> Result<CollectibleTransaction, Error> {
        collectible::sign_collectible_request_with_pin(request_id, pin_hex, &self.safe_user).await
    }

    pub async fn cancel_collectible_request(
        &self,
        request_id: &str,
        pin_base64: &str,
    ) -> Result<CollectibleTransaction, Error> {
        collectible::cancel_collectible_request(request_id, pin_base64, &self.safe_user).await
    }

    pub async fn cancel_collectible_request_with_pin(
        &self,
        request_id: &str,
        pin_hex: &str,
    ) -> Result<CollectibleTransaction, Error> {
        collectible::cancel_collectible_request_with_pin(request_id, pin_hex, &self.safe_user).await
    }

    pub async fn unlock_collectible_request(
        &self,
        request_id: &str,
        pin_base64: &str,
    ) -> Result<CollectibleTransaction, Error> {
        collectible::unlock_collectible_request(request_id, pin_base64, &self.safe_user).await
    }

    pub async fn unlock_collectible_request_with_pin(
        &self,
        request_id: &str,
        pin_hex: &str,
    ) -> Result<CollectibleTransaction, Error> {
        collectible::unlock_collectible_request_with_pin(request_id, pin_hex, &self.safe_user).await
    }

    pub async fn verify_transactions(
        &self,
        requests: &[TransactionRequest],
    ) -> Result<Vec<TransactionView>, Error> {
        transaction::verify_transactions(requests, &self.safe_user).await
    }

    pub async fn create_transaction_request(
        &self,
        request_id: &str,
        raw: &str,
    ) -> Result<TransactionView, Error> {
        transaction::create_transaction_request(request_id, raw, &self.safe_user).await
    }

    pub async fn send_transactions(
        &self,
        requests: &[TransactionRequest],
    ) -> Result<Vec<TransactionView>, Error> {
        transaction::send_transactions(requests, &self.safe_user).await
    }

    pub async fn submit_transaction(
        &self,
        request_id: &str,
        signed_raw: &str,
    ) -> Result<TransactionView, Error> {
        transaction::submit_transaction(request_id, signed_raw, &self.safe_user).await
    }

    pub async fn get_transaction(&self, request_id: &str) -> Result<TransactionView, Error> {
        transaction::get_transaction(request_id, &self.safe_user).await
    }

    pub async fn create_safe_multisig_requests(
        &self,
        requests: &[TransactionRequest],
    ) -> Result<Vec<SafeMultisigRequest>, Error> {
        safe_multisig::create_safe_multisig_requests(requests, &self.safe_user).await
    }

    pub async fn create_safe_multisig_request(
        &self,
        request_id: &str,
        raw: &str,
    ) -> Result<SafeMultisigRequest, Error> {
        safe_multisig::create_safe_multisig_request(request_id, raw, &self.safe_user).await
    }

    pub async fn fetch_safe_multisig_request(
        &self,
        id_or_hash: &str,
    ) -> Result<SafeMultisigRequest, Error> {
        safe_multisig::fetch_safe_multisig_request(id_or_hash, &self.safe_user).await
    }

    pub async fn sign_safe_multisig_request(
        &self,
        id_or_hash: &str,
        signed_raw: &str,
    ) -> Result<SafeMultisigRequest, Error> {
        safe_multisig::sign_safe_multisig_request(id_or_hash, signed_raw, &self.safe_user).await
    }

    pub async fn fetch_sign_safe_multisig_request(
        &self,
        id_or_hash: &str,
    ) -> Result<SafeMultisigRequest, Error> {
        safe_multisig::fetch_sign_safe_multisig_request(id_or_hash, &self.safe_user).await
    }

    pub async fn unlock_safe_multisig_request(
        &self,
        id_or_hash: &str,
    ) -> Result<SafeMultisigRequest, Error> {
        safe_multisig::unlock_safe_multisig_request(id_or_hash, &self.safe_user).await
    }

    pub async fn cancel_safe_multisig_request(
        &self,
        id_or_hash: &str,
    ) -> Result<SafeMultisigRequest, Error> {
        safe_multisig::cancel_safe_multisig_request(id_or_hash, &self.safe_user).await
    }

    pub async fn list_legacy_multisigs(
        &self,
        limit: u32,
        offset: Option<&str>,
    ) -> Result<Vec<LegacyMultisigUtxo>, Error> {
        legacy_multisig::list_legacy_multisigs(limit, offset, &self.safe_user).await
    }

    pub async fn list_multisig_outputs(
        &self,
        query: &LegacyMultisigQuery,
    ) -> Result<Vec<LegacyMultisigUtxo>, Error> {
        legacy_multisig::list_multisig_outputs(query, &self.safe_user).await
    }

    pub async fn list_multisig_outputs_for_members<T, I>(
        &self,
        members: I,
        threshold: u8,
        state: Option<&str>,
        offset: Option<&str>,
        limit: Option<u32>,
    ) -> Result<Vec<LegacyMultisigUtxo>, Error>
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        legacy_multisig::list_multisig_outputs_for_members(
            members,
            threshold,
            state,
            offset,
            limit,
            &self.safe_user,
        )
        .await
    }

    pub async fn create_multisig(
        &self,
        action: &str,
        raw: &str,
    ) -> Result<LegacyMultisigRequest, Error> {
        legacy_multisig::create_multisig(action, raw, &self.safe_user).await
    }

    pub async fn sign_multisig(
        &self,
        request_id: &str,
        pin_base64: &str,
    ) -> Result<LegacyMultisigRequest, Error> {
        legacy_multisig::sign_multisig(request_id, pin_base64, &self.safe_user).await
    }

    pub async fn sign_multisig_with_pin(
        &self,
        request_id: &str,
        pin_hex: &str,
    ) -> Result<LegacyMultisigRequest, Error> {
        legacy_multisig::sign_multisig_with_pin(request_id, pin_hex, &self.safe_user).await
    }

    pub async fn unlock_multisig(&self, request_id: &str, pin_base64: &str) -> Result<(), Error> {
        legacy_multisig::unlock_multisig(request_id, pin_base64, &self.safe_user).await
    }

    pub async fn unlock_multisig_with_pin(
        &self,
        request_id: &str,
        pin_hex: &str,
    ) -> Result<(), Error> {
        legacy_multisig::unlock_multisig_with_pin(request_id, pin_hex, &self.safe_user).await
    }

    pub async fn cancel_multisig(&self, request_id: &str) -> Result<(), Error> {
        legacy_multisig::cancel_multisig(request_id, &self.safe_user).await
    }

    pub async fn read_collection(&self, collection_hash: &str) -> Result<Collection, Error> {
        inscription::read_collection(collection_hash).await
    }

    pub async fn read_inscription(&self, inscription_hash: &str) -> Result<Inscription, Error> {
        inscription::read_inscription(inscription_hash).await
    }

    pub async fn read_collection_items(
        &self,
        collection_hash: &str,
    ) -> Result<Vec<Inscription>, Error> {
        inscription::read_collection_items(collection_hash).await
    }

    pub fn encode_inscription_deploy(&self, deploy: &InscriptionDeploy) -> Result<Vec<u8>, Error> {
        inscription::encode_inscription_deploy(deploy)
    }

    pub fn encode_inscription_inscribe(
        &self,
        inscribe: &InscriptionInscribe,
    ) -> Result<Vec<u8>, Error> {
        inscription::encode_inscription_inscribe(inscribe)
    }

    pub fn encode_inscription_distribute(
        &self,
        distribute: &InscriptionDistribute,
    ) -> Result<Vec<u8>, Error> {
        inscription::encode_inscription_distribute(distribute)
    }

    pub fn encode_inscription_occupy(&self, occupy: &InscriptionOccupy) -> Result<Vec<u8>, Error> {
        inscription::encode_inscription_occupy(occupy)
    }

    pub async fn get_computer_info(&self) -> Result<ComputerInfo, Error> {
        computer::get_computer_info().await
    }

    pub async fn get_computer_user(&self, address: &str) -> Result<Option<ComputerUser>, Error> {
        computer::get_computer_user(address).await
    }

    pub async fn get_computer_deployed_assets(&self) -> Result<Vec<ComputerDeployedAsset>, Error> {
        computer::get_computer_deployed_assets().await
    }

    pub async fn get_computer_system_call(
        &self,
        id: &str,
    ) -> Result<Option<ComputerSystemCallResponse>, Error> {
        computer::get_computer_system_call(id).await
    }

    pub async fn computer_deploy_external_assets(&self, asset_ids: &[String]) -> Result<(), Error> {
        computer::computer_deploy_external_assets(asset_ids).await
    }

    pub async fn lock_computer_nonce_account(
        &self,
        mix: &str,
    ) -> Result<ComputerNonceAccount, Error> {
        computer::lock_computer_nonce_account(mix).await
    }

    pub async fn get_fee_on_xin_based_on_sol(
        &self,
        sol_amount: &str,
    ) -> Result<ComputerFee, Error> {
        computer::get_fee_on_xin_based_on_sol(sol_amount).await
    }

    pub async fn preview_register_computer(&self) -> Result<ComputerRegisterPreview, Error> {
        computer::preview_register_computer(&self.safe_user).await
    }

    pub async fn register_computer(&self) -> Result<TransactionView, Error> {
        computer::register_computer(&self.safe_user).await
    }

    pub async fn connect_blaze(&self) -> Result<BlazeClient, Error> {
        blaze::connect_blaze(&self.safe_user).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_safe_user() -> SafeUser {
        SafeUser {
            user_id: "user-id".to_string(),
            session_id: "session-id".to_string(),
            session_private_key: "session-private-key".to_string(),
            server_public_key: "server-public-key".to_string(),
            spend_private_key: "spend-private-key".to_string(),
            is_spend_private_sum: false,
        }
    }

    #[test]
    fn test_client_keeps_safe_user_identity() {
        let client = Client::new(test_safe_user());
        assert_eq!(client.user_id(), "user-id");
        assert_eq!(client.session_id(), "session-id");
        assert_eq!(client.safe_user().user_id, "user-id");
    }

    #[test]
    fn test_client_into_safe_user() {
        let client = Client::new(test_safe_user());
        let safe_user = client.into_safe_user();
        assert_eq!(safe_user.user_id, "user-id");
    }
}
