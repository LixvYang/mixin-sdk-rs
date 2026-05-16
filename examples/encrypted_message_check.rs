use mixin_sdk_rs::app;
use mixin_sdk_rs::error::Error;
use mixin_sdk_rs::message::{MessageSession, encrypted_text_message_request, post_message};
use mixin_sdk_rs::safe::SafeUser;
use mixin_sdk_rs::session::fetch_user_sessions;

#[derive(Debug)]
struct Args {
    keystore: String,
    recipient: String,
    text: String,
    send: bool,
    recipient_decrypts: bool,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = parse_args()?;
    let user = SafeUser::new_from_file(&args.keystore)?;
    let recipient = resolve_recipient(&args.recipient, &user).await?;
    let sessions = fetch_user_sessions(std::slice::from_ref(&recipient), &user).await?;
    let message_sessions: Vec<MessageSession> = sessions.iter().map(MessageSession::from).collect();
    if message_sessions.is_empty() {
        return Err(Error::DataNotFound("recipient sessions".to_string()));
    }

    let message = encrypted_text_message_request(
        &user.user_id,
        &recipient,
        &args.text,
        &message_sessions,
        &user,
    )?;
    println!("recipient_sessions: {}", message_sessions.len());
    println!("message_id: {}", message.message_id);
    println!("conversation_id: {}", message.conversation_id);
    println!("category: {}", message.category);
    println!("encrypted_data_base64_len: {}", message.data_base64.len());

    if args.send && !args.recipient_decrypts {
        return Err(Error::Input(
            "--send requires --recipient-decrypts because normal Messenger clients show encrypted payloads as unreadable text".to_string(),
        ));
    }

    if args.send {
        post_message(message, &user).await?;
        println!("sent: true");
    } else {
        println!("sent: false");
    }

    Ok(())
}

async fn resolve_recipient(recipient: &str, safe_user: &SafeUser) -> Result<String, Error> {
    if recipient == "app-creator" || recipient == "creator" {
        let app = app::get_app(&safe_user.user_id, safe_user).await?;
        return app
            .creator_id
            .ok_or_else(|| Error::DataNotFound("app response is missing creator_id".to_string()));
    }
    Ok(recipient.to_string())
}

fn parse_args() -> Result<Args, Error> {
    let mut keystore = None;
    let mut recipient = None;
    let mut text = "hello from rust sdk encrypted message".to_string();
    let mut send = false;
    let mut recipient_decrypts = false;

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
            "--send" => {
                send = true;
            }
            "--recipient-decrypts" => {
                recipient_decrypts = true;
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
        send,
        recipient_decrypts,
    })
}

fn print_usage() {
    eprintln!(
        "usage: cargo run --example encrypted_message_check -- --keystore <path> --recipient <user-id|app-creator> [--text <message>] [--send --recipient-decrypts]"
    );
}
