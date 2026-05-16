use mixin_sdk_rs::blaze::connect_blaze;
use mixin_sdk_rs::error::Error;
use mixin_sdk_rs::safe::SafeUser;
use std::time::Duration;

#[derive(Debug)]
struct Args {
    keystore: String,
    timeout_seconds: u64,
    ack: bool,
    connect_only: bool,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = parse_args()?;
    let user = SafeUser::new_from_file(&args.keystore)?;
    let mut blaze = connect_blaze(&user).await?;

    let request_id = blaze.list_pending_messages().await?;
    if args.connect_only {
        println!("list_pending_request_id: {}", request_id);
        blaze.close().await?;
        return Ok(());
    }

    let message = tokio::time::timeout(
        Duration::from_secs(args.timeout_seconds),
        blaze.next_message(),
    )
    .await
    .map_err(|_| Error::Input("timed out waiting for a Blaze message".to_string()))??;

    println!("message_id: {}", message.message_id);
    println!("conversation_id: {}", message.conversation_id);
    println!("user_id: {}", message.user_id);
    println!("category: {}", message.category);
    if args.ack {
        blaze.mark_message_read(&message.message_id).await?;
        println!("ack: READ");
    }

    blaze.close().await?;
    Ok(())
}

fn parse_args() -> Result<Args, Error> {
    let mut keystore = None;
    let mut timeout_seconds = 10;
    let mut ack = false;
    let mut connect_only = false;

    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--keystore" => {
                keystore = Some(
                    iter.next()
                        .ok_or_else(|| Error::Input("--keystore requires a path".to_string()))?,
                );
            }
            "--timeout-seconds" => {
                let value = iter.next().ok_or_else(|| {
                    Error::Input("--timeout-seconds requires a number".to_string())
                })?;
                timeout_seconds = value
                    .parse()
                    .map_err(|_| Error::Input("--timeout-seconds must be a number".to_string()))?;
            }
            "--ack" => {
                ack = true;
            }
            "--connect-only" => {
                connect_only = true;
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => return Err(Error::Input(format!("unknown argument: {other}"))),
        }
    }

    let keystore = keystore.ok_or_else(|| {
        print_usage();
        Error::Input("--keystore is required".to_string())
    })?;

    Ok(Args {
        keystore,
        timeout_seconds,
        ack,
        connect_only,
    })
}

fn print_usage() {
    eprintln!(
        "usage: cargo run --example blaze_listen_once -- --keystore <path> [--connect-only] [--timeout-seconds 10] [--ack]"
    );
}
