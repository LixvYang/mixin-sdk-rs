use mixin_sdk_rs::Client;
use mixin_sdk_rs::error::Error;

#[derive(Debug)]
struct Args {
    keystore: String,
    recipient: String,
    text: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = parse_args()?;
    let client = Client::from_keystore_file(&args.keystore)?;
    let recipient = resolve_recipient(&args.recipient, &client).await?;

    let message = client.send_text_message(&recipient, &args.text).await?;
    println!("message_id: {}", message.message_id);
    println!("conversation_id: {}", message.conversation_id);
    Ok(())
}

async fn resolve_recipient(recipient: &str, client: &Client) -> Result<String, Error> {
    if recipient == "app-creator" || recipient == "creator" {
        return client.app_creator_id().await;
    }
    Ok(recipient.to_string())
}

fn parse_args() -> Result<Args, Error> {
    let mut keystore = None;
    let mut recipient = None;
    let mut text = "hello from rust sdk".to_string();

    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--keystore" => {
                keystore = Some(
                    iter.next()
                        .ok_or_else(|| Error::Input("--keystore requires a path".to_string()))?,
                );
            }
            "--recipient" => {
                recipient =
                    Some(iter.next().ok_or_else(|| {
                        Error::Input("--recipient requires a user id".to_string())
                    })?);
            }
            "--text" => {
                text = iter
                    .next()
                    .ok_or_else(|| Error::Input("--text requires content".to_string()))?;
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
    let recipient = recipient.ok_or_else(|| {
        print_usage();
        Error::Input("--recipient is required".to_string())
    })?;

    Ok(Args {
        keystore,
        recipient,
        text,
    })
}

fn print_usage() {
    eprintln!(
        "usage: cargo run --example send_message -- --keystore <path> --recipient <user-id|app-creator> [--text <message>]"
    );
}
