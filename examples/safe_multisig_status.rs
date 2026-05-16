use mixin_sdk_rs::error::Error;
use mixin_sdk_rs::safe::SafeUser;
use mixin_sdk_rs::safe_multisig::fetch_safe_multisig_request;

#[derive(Debug, Default)]
struct Args {
    keystore: String,
    id: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = parse_args()?;
    let safe_user = SafeUser::new_from_file(&args.keystore)?;
    let request = fetch_safe_multisig_request(&args.id, &safe_user).await?;
    println!("request_id: {:?}", request.request_id);
    println!("transaction_hash: {:?}", request.transaction_hash);
    println!("asset_id: {:?}", request.asset_id);
    println!("amount: {:?}", request.amount);
    println!("senders: {}", request.senders.len());
    println!("signers: {}", request.signers.len());
    println!("views: {}", request.views.len());
    println!("has_raw_transaction: {}", request.raw_transaction.is_some());
    Ok(())
}

fn parse_args() -> Result<Args, Error> {
    let mut args = Args::default();
    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--keystore" => args.keystore = next_value(&mut iter, "--keystore")?,
            "--id" => args.id = next_value(&mut iter, "--id")?,
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => return Err(Error::Input(format!("unknown argument: {other}"))),
        }
    }
    if args.keystore.is_empty() || args.id.is_empty() {
        print_usage();
        return Err(Error::Input("--keystore and --id are required".to_string()));
    }
    Ok(args)
}

fn next_value(iter: &mut impl Iterator<Item = String>, name: &str) -> Result<String, Error> {
    iter.next()
        .ok_or_else(|| Error::Input(format!("{name} requires a value")))
}

fn print_usage() {
    eprintln!(
        "usage: cargo run --example safe_multisig_status -- --keystore <path> --id <request-id-or-hash>"
    );
}
