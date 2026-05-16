use mixin_sdk_rs::app;
use mixin_sdk_rs::error::Error;
use mixin_sdk_rs::mix_address::MixAddress;
use mixin_sdk_rs::output::list_unspent_outputs;
use mixin_sdk_rs::safe::SafeUser;
use mixin_sdk_rs::safe_transaction::{
    SafeTransactionRecipient, build_safe_transaction, encode_unsigned_safe_transaction,
    get_unspent_outputs_for_recipients, normalize_asset_id, request_ghost_recipients_with_trace_id,
    sign_safe_transaction_with_index,
};
use mixin_sdk_rs::transaction::{TransactionRequest, send_transactions, verify_transactions};
use mixin_sdk_rs::utils::hash_members;
use uuid::Uuid;

#[derive(Debug, Default)]
struct Args {
    keystore: String,
    asset: String,
    receiver: String,
    amount: String,
    trace: Option<String>,
    extra: Vec<u8>,
    references: Vec<String>,
    send: bool,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = parse_args()?;
    let safe_user = SafeUser::new_from_file(&args.keystore)?;
    let trace = args.trace.unwrap_or_else(|| Uuid::new_v4().to_string());
    let asset = normalize_asset_id(&args.asset)?;
    let receiver = resolve_receiver(&args.receiver, &safe_user).await?;
    let mut recipients = vec![recipient_from_arg(&receiver, &args.amount)?];

    let members_hash = hash_members([safe_user.user_id.as_str()]);
    let outputs = list_unspent_outputs(&members_hash, 1, Some(&asset), &safe_user).await?;
    let (input_count, change) = get_unspent_outputs_for_recipients(&outputs, &recipients)?;
    let selected = &outputs[..input_count];
    if change != "0" {
        let change_address = MixAddress::new_uuid(vec![safe_user.user_id.clone()], 1)?;
        recipients.push(SafeTransactionRecipient::mix_address(
            change_address,
            change,
        ));
    }

    let ghosts = request_ghost_recipients_with_trace_id(&recipients, &trace, &safe_user).await?;
    let tx = build_safe_transaction(
        selected,
        &recipients,
        &ghosts,
        args.extra,
        args.references.clone(),
    )?;
    let raw = encode_unsigned_safe_transaction(&tx)?;
    let verified = verify_transactions(
        &[TransactionRequest {
            request_id: trace.clone(),
            raw,
        }],
        &safe_user,
    )
    .await?;
    let verified = one(verified)?;
    let views = verified
        .views
        .clone()
        .ok_or_else(|| Error::DataNotFound("sequencer response is missing views".to_string()))?;
    let signed_raw = sign_safe_transaction_with_index(
        &tx,
        &views,
        &safe_user.spend_private_key,
        safe_user.is_spend_private_sum,
        0,
    )?;

    println!("trace: {trace}");
    println!("receiver: {receiver}");
    println!("verify_state: {:?}", verified.state);
    println!("transaction_hash: {:?}", verified.transaction_hash);
    println!("inputs: {}", tx.inputs.len());
    println!("outputs: {}", tx.outputs.len());
    println!("views: {}", views.len());
    println!("signed_raw_bytes: {}", signed_raw.len() / 2);

    if args.send {
        let sent = send_transactions(
            &[TransactionRequest {
                request_id: trace,
                raw: signed_raw,
            }],
            &safe_user,
        )
        .await?;
        let sent = one(sent)?;
        println!("send_state: {:?}", sent.state);
        println!("snapshot_id: {:?}", sent.snapshot_id);
    } else {
        println!("send_skipped: pass --send to submit the signed transaction");
    }

    Ok(())
}

async fn resolve_receiver(receiver: &str, safe_user: &SafeUser) -> Result<String, Error> {
    if receiver == "app-creator" || receiver == "creator" {
        let app = app::get_app(&safe_user.user_id, safe_user).await?;
        return app
            .creator_id
            .ok_or_else(|| Error::DataNotFound("app response is missing creator_id".to_string()));
    }
    Ok(receiver.to_string())
}

fn recipient_from_arg(receiver: &str, amount: &str) -> Result<SafeTransactionRecipient, Error> {
    let mix_address = if receiver.starts_with("MIX") {
        MixAddress::parse(receiver)?
    } else if receiver.starts_with("XIN") {
        MixAddress::new_mainnet(vec![receiver.to_string()], 1)?
    } else {
        MixAddress::new_uuid(vec![receiver.to_string()], 1)?
    };
    Ok(SafeTransactionRecipient::mix_address(mix_address, amount))
}

fn one<T>(mut values: Vec<T>) -> Result<T, Error> {
    if values.len() != 1 {
        return Err(Error::DataNotFound(format!(
            "expected one item, got {}",
            values.len()
        )));
    }
    Ok(values.remove(0))
}

fn parse_args() -> Result<Args, Error> {
    let mut args = Args::default();
    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--keystore" => args.keystore = next_value(&mut iter, "--keystore")?,
            "--asset" => args.asset = next_value(&mut iter, "--asset")?,
            "--receiver" => args.receiver = next_value(&mut iter, "--receiver")?,
            "--amount" => args.amount = next_value(&mut iter, "--amount")?,
            "--trace" => args.trace = Some(next_value(&mut iter, "--trace")?),
            "--extra-text" => args.extra = next_value(&mut iter, "--extra-text")?.into_bytes(),
            "--extra-hex" => args.extra = hex::decode(next_value(&mut iter, "--extra-hex")?)?,
            "--reference" => args.references.push(next_value(&mut iter, "--reference")?),
            "--send" => args.send = true,
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => return Err(Error::Input(format!("unknown argument: {other}"))),
        }
    }

    if args.keystore.is_empty()
        || args.asset.is_empty()
        || args.receiver.is_empty()
        || args.amount.is_empty()
    {
        print_usage();
        return Err(Error::Input(
            "--keystore, --asset, --receiver, and --amount are required".to_string(),
        ));
    }
    Ok(args)
}

fn next_value(iter: &mut impl Iterator<Item = String>, name: &str) -> Result<String, Error> {
    iter.next()
        .ok_or_else(|| Error::Input(format!("{name} requires a value")))
}

fn print_usage() {
    eprintln!(
        "usage: cargo run --example safe_transfer_verify -- --keystore <path> --asset <asset-id-or-kernel-hash> --receiver <uuid|MIX|XIN|app-creator> --amount <amount> [--trace <uuid>] [--extra-text <text>|--extra-hex <hex>] [--reference <hash>] [--send]"
    );
}
