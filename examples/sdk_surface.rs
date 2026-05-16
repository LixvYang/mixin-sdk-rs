use mixin_sdk_rs::{
    app::{AppRequest, AppSafeRegistrationRequest, AppSafeSessionRequest},
    circle::CircleConversationQuery,
    collectible::{COLLECTIBLE_ACTION_SIGN, CollectibleOutputsRequest},
    computer::{
        OPERATION_TYPE_SYSTEM_CALL, build_system_call_extra, check_system_call_size,
        encode_mtg_extra, encode_operation_memo,
    },
    deposit::{DepositEntryRequest, SafePendingDepositQuery},
    external::{ExternalProxyRequest, ExternalTransactionQuery},
    inscription::{
        INSCRIPTION_MODE_INSTANT, InscriptionDeploy, InscriptionDistribute, InscriptionInscribe,
        InscriptionOccupy, encode_inscription_deploy, encode_inscription_distribute,
        encode_inscription_inscribe, encode_inscription_occupy,
    },
    legacy_multisig,
    legacy_transfer::LegacyTransferRequest,
    oauth::{self, AccessTokenRequest, AuthorizeRequest},
    payment::{PaymentRequest, TransferPaymentRequest},
    snapshot::{SafeSnapshotNotificationRequest, SafeSnapshotQuery},
    user::{LogQuery, PreferenceRequest, RELATIONSHIP_ACTION_ADD, RelationshipRequest},
};

fn print_json<T: serde::Serialize>(label: &str, value: &T) -> Result<(), serde_json::Error> {
    println!("{label}: {}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oauth_token = AccessTokenRequest {
        client_id: "client-id".to_string(),
        code: "authorization-code".to_string(),
        ed25519: Some("client-public-ed25519".to_string()),
        client_secret: None,
        code_verifier: Some("pkce-verifier".to_string()),
    };
    print_json("oauth token request", &oauth_token)?;

    let authorize = AuthorizeRequest {
        authorization_id: "authorization-id".to_string(),
        scopes: vec![
            oauth::SCOPE_PROFILE_READ.to_string(),
            oauth::SCOPE_ASSETS_READ.to_string(),
        ],
        pin_base64: None,
    };
    print_json("oauth authorize request", &authorize)?;

    let payment = PaymentRequest::Transfer(TransferPaymentRequest {
        asset_id: "asset-id".to_string(),
        opponent_id: "opponent-user-id".to_string(),
        amount: Some("0.01".to_string()),
        trace_id: Some("trace-id".to_string()),
        memo: Some("demo".to_string()),
        pin: None,
    });
    print_json("payment request", &payment)?;

    let deposit = DepositEntryRequest {
        chain_id: "chain-id".to_string(),
        members: Some(vec!["member-a".to_string(), "member-b".to_string()]),
        threshold: Some(2),
    };
    print_json("deposit entry request", &deposit)?;

    let app = AppRequest {
        redirect_uri: "https://example.com/oauth".to_string(),
        home_uri: "https://example.com".to_string(),
        name: "Example".to_string(),
        description: "Example app".to_string(),
        icon_base64: "base64-icon".to_string(),
        category: "TOOLS".to_string(),
        capabilities: vec!["CONTACT".to_string()],
        resource_patterns: vec!["https://example.com/*".to_string()],
    };
    print_json("app request", &app)?;
    print_json(
        "app safe session request",
        &AppSafeSessionRequest {
            session_public_key: "session-public-key".to_string(),
        },
    )?;
    print_json(
        "app safe registration request",
        &AppSafeRegistrationRequest {
            spend_public_key: "spend-public-key".to_string(),
            signature_base64: "signature".to_string(),
        },
    )?;

    print_json(
        "preference request",
        &PreferenceRequest {
            receive_message_source: Some("EVERYBODY".to_string()),
            fiat_currency: Some("USD".to_string()),
            ..Default::default()
        },
    )?;
    print_json(
        "relationship request",
        &RelationshipRequest {
            user_id: "user-id".to_string(),
            action: RELATIONSHIP_ACTION_ADD.to_string(),
            phone: Some("+10000000000".to_string()),
            full_name: Some("Alice".to_string()),
        },
    )?;
    print_json(
        "logs query",
        &LogQuery {
            category: Some("PIN".to_string()),
            limit: Some(20),
            ..Default::default()
        },
    )?;

    let legacy_transfer = LegacyTransferRequest {
        asset_id: "asset-id".to_string(),
        opponent_id: "opponent-user-id".to_string(),
        amount: Some("0.01".to_string()),
        trace_id: Some("trace-id".to_string()),
        memo: Some("legacy demo".to_string()),
        pin: Some("<encrypted-pin-base64url>".to_string()),
    };
    print_json("legacy transfer request", &legacy_transfer)?;

    let hash = legacy_multisig::legacy_multisig_members_hash(["member-b", "member-a"]);
    println!("legacy multisig members hash: {hash}");

    print_json(
        "collectible outputs request",
        &CollectibleOutputsRequest {
            members: vec!["member-a".to_string(), "member-b".to_string()],
            threshold: 2,
            state: Some("unspent".to_string()),
            ..Default::default()
        },
    )?;
    println!("collectible transfer action: {COLLECTIBLE_ACTION_SIGN}");

    print_json(
        "circle conversation query",
        &CircleConversationQuery {
            offset: Some("2026-05-16T00:00:00Z".to_string()),
            limit: Some(50),
        },
    )?;
    print_json(
        "pending safe deposit query",
        &SafePendingDepositQuery {
            asset: Some("asset-id".to_string()),
            limit: Some(20),
            ..Default::default()
        },
    )?;
    print_json(
        "external transactions query",
        &ExternalTransactionQuery {
            asset: Some("asset-id".to_string()),
            order: Some("DESC".to_string()),
            limit: Some(20),
            ..Default::default()
        },
    )?;
    print_json(
        "external proxy request",
        &ExternalProxyRequest {
            method: "sendrawtransaction".to_string(),
            params: vec![serde_json::json!("raw-transaction")],
        },
    )?;
    print_json(
        "safe snapshots query",
        &SafeSnapshotQuery {
            asset: Some("asset-id".to_string()),
            limit: Some(20),
            ..Default::default()
        },
    )?;
    print_json(
        "safe snapshot notification",
        &SafeSnapshotNotificationRequest {
            transaction_hash: "transaction-hash".to_string(),
            output_index: 0,
            receiver_id: "receiver-id".to_string(),
        },
    )?;

    let deploy = InscriptionDeploy::new(
        INSCRIPTION_MODE_INSTANT,
        "1000000",
        "1000000000",
        "MAO",
        "Mao Demo",
        "image/webp;base64,AAAA",
    );
    println!(
        "inscription deploy extra: {}",
        String::from_utf8(encode_inscription_deploy(&deploy)?)?
    );
    println!(
        "inscription inscribe extra: {}",
        String::from_utf8(encode_inscription_inscribe(&InscriptionInscribe::new(
            "MIX-recipient",
            Some("text/plain;charset=UTF-8,hello".to_string()),
        ))?)?
    );
    println!(
        "inscription distribute extra: {}",
        String::from_utf8(encode_inscription_distribute(&InscriptionDistribute::new(
            0
        ))?)?
    );
    println!(
        "inscription occupy extra: {}",
        String::from_utf8(encode_inscription_occupy(&InscriptionOccupy::new(0))?)?
    );

    let system_call_extra = build_system_call_extra(
        "1",
        "00000000-0000-0000-0000-000000000002",
        false,
        Some("00000000-0000-0000-0000-000000000003"),
    )?;
    let computer_memo = encode_operation_memo(OPERATION_TYPE_SYSTEM_CALL, &system_call_extra);
    let computer_extra = encode_mtg_extra("00000000-0000-0000-0000-000000000001", &computer_memo)?;
    println!("computer mtg extra: {computer_extra}");
    println!(
        "computer system call size ok: {}",
        check_system_call_size(&vec![0; 1232])
    );

    Ok(())
}
