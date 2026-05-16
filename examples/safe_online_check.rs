use mixin_sdk_rs::error::Error;
use mixin_sdk_rs::output::{fetch_safe_outputs, list_unspent_outputs};
use mixin_sdk_rs::safe::SafeUser;
use mixin_sdk_rs::user;
use mixin_sdk_rs::utils::hash_members;

#[derive(Debug, Default)]
struct Args {
    keystore: String,
    asset: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = parse_args()?;
    let safe_user = SafeUser::new_from_file(&args.keystore)?;

    let me = user::request_user_me(&safe_user).await?;
    println!("user_id: {}", me.user_id);

    let members_hash = hash_members([safe_user.user_id.as_str()]);
    let outputs = list_unspent_outputs(&members_hash, 1, args.asset.as_deref(), &safe_user).await?;
    println!("unspent_outputs: {}", outputs.len());
    for (index, output) in outputs.iter().take(5).enumerate() {
        println!(
            "output[{}]: id={} hash={} index={:?} asset_id={:?} kernel_asset_id={:?} amount={:?}",
            index,
            output.output_id,
            output.transaction_hash.as_deref().unwrap_or(""),
            output.output_index,
            output.asset_id,
            output.kernel_asset_id,
            output.amount
        );
    }

    let ids: Vec<String> = outputs
        .iter()
        .take(2)
        .map(|output| output.output_id.clone())
        .collect();
    if !ids.is_empty() {
        let fetched = fetch_safe_outputs(&safe_user.user_id, &ids, &safe_user).await?;
        println!("fetched_outputs: {}", fetched.len());
    }

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
            "--asset" => {
                args.asset = Some(
                    iter.next()
                        .ok_or_else(|| Error::Input("--asset requires an asset".to_string()))?,
                );
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => {
                return Err(Error::Input(format!("unknown argument: {other}")));
            }
        }
    }

    if args.keystore.is_empty() {
        print_usage();
        return Err(Error::Input("--keystore is required".to_string()));
    }
    Ok(args)
}

fn print_usage() {
    eprintln!(
        "usage: cargo run --example safe_online_check -- --keystore <path> [--asset <asset-id-or-kernel-hash>]"
    );
}
