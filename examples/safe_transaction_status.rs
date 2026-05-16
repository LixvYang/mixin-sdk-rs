use mixin_sdk_rs::error::Error;
use mixin_sdk_rs::safe::SafeUser;
use mixin_sdk_rs::transaction::get_transaction;

#[derive(Debug, Default)]
struct Args {
    keystore: String,
    request_id: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = parse_args()?;
    let safe_user = SafeUser::new_from_file(&args.keystore)?;
    let tx = get_transaction(&args.request_id, &safe_user).await?;
    println!("request_id: {:?}", tx.request_id);
    println!("transaction_hash: {:?}", tx.transaction_hash);
    println!("state: {:?}", tx.state);
    println!("snapshot_id: {:?}", tx.snapshot_id);
    println!("snapshot_hash: {:?}", tx.snapshot_hash);
    println!("snapshot_at: {:?}", tx.snapshot_at);
    Ok(())
}

fn parse_args() -> Result<Args, Error> {
    let mut args = Args::default();
    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--keystore" => {
                args.keystore = iter
                    .next()
                    .ok_or_else(|| Error::Input("--keystore requires a path".to_string()))?;
            }
            "--request-id" => {
                args.request_id = iter
                    .next()
                    .ok_or_else(|| Error::Input("--request-id requires a value".to_string()))?;
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => return Err(Error::Input(format!("unknown argument: {other}"))),
        }
    }
    if args.keystore.is_empty() || args.request_id.is_empty() {
        print_usage();
        return Err(Error::Input(
            "--keystore and --request-id are required".to_string(),
        ));
    }
    Ok(args)
}

fn print_usage() {
    eprintln!(
        "usage: cargo run --example safe_transaction_status -- --keystore <path> --request-id <uuid>"
    );
}
