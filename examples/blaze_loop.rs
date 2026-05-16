use mixin_sdk_rs::blaze::{
    BlazeClient, BlazeListener, BlazeLoopOptions, BlazeMessageKind, MessageView, run_blaze_loop,
};
use mixin_sdk_rs::error::Error;
use mixin_sdk_rs::safe::SafeUser;
use std::time::Duration;

#[derive(Debug)]
struct Args {
    keystore: String,
    timeout_seconds: u64,
    ack: bool,
    max_reconnects: Option<usize>,
    show_data: bool,
}

struct PrintListener {
    sync_ack: bool,
    show_data: bool,
}

#[async_trait::async_trait]
impl BlazeListener for PrintListener {
    async fn on_message(
        &mut self,
        _client: &mut BlazeClient,
        message: MessageView,
        _client_user_id: &str,
    ) -> Result<(), Error> {
        print_message("message", &message, self.show_data);
        Ok(())
    }

    async fn on_ack_receipt(
        &mut self,
        _client: &mut BlazeClient,
        message: MessageView,
        _client_user_id: &str,
    ) -> Result<(), Error> {
        print_message("ack_receipt", &message, self.show_data);
        Ok(())
    }

    async fn on_transfer(
        &mut self,
        _client: &mut BlazeClient,
        message: MessageView,
        _client_user_id: &str,
    ) -> Result<(), Error> {
        print_message("transfer", &message, self.show_data);
        Ok(())
    }

    async fn on_conversation(
        &mut self,
        _client: &mut BlazeClient,
        message: MessageView,
        _client_user_id: &str,
    ) -> Result<(), Error> {
        print_message("conversation", &message, self.show_data);
        Ok(())
    }

    async fn on_safe_snapshot(
        &mut self,
        _client: &mut BlazeClient,
        message: MessageView,
        _client_user_id: &str,
    ) -> Result<(), Error> {
        print_message("safe_snapshot", &message, self.show_data);
        Ok(())
    }

    async fn on_safe_inscription(
        &mut self,
        _client: &mut BlazeClient,
        message: MessageView,
        _client_user_id: &str,
    ) -> Result<(), Error> {
        print_message("safe_inscription", &message, self.show_data);
        Ok(())
    }

    async fn on_disconnect(&mut self, error: &str, attempt: usize) -> Result<(), Error> {
        eprintln!("blaze disconnected: attempt={} error={}", attempt, error);
        Ok(())
    }

    fn sync_ack(&self) -> bool {
        self.sync_ack
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = parse_args()?;
    let user = SafeUser::new_from_file(&args.keystore)?;
    let mut listener = PrintListener {
        sync_ack: args.ack,
        show_data: args.show_data,
    };
    let options = BlazeLoopOptions {
        sync_ack: args.ack,
        max_reconnects: args.max_reconnects,
        ..BlazeLoopOptions::default()
    };

    if args.timeout_seconds == 0 {
        run_blaze_loop(&user, &mut listener, options).await
    } else {
        match tokio::time::timeout(
            Duration::from_secs(args.timeout_seconds),
            run_blaze_loop(&user, &mut listener, options),
        )
        .await
        {
            Ok(result) => result,
            Err(_) => {
                println!("timeout: {}s", args.timeout_seconds);
                Ok(())
            }
        }
    }
}

fn print_message(label: &str, message: &MessageView, show_data: bool) {
    println!(
        "{}: id={} conversation_id={} user_id={} category={} kind={:?}",
        label,
        message.message_id,
        message.conversation_id,
        message.user_id,
        message.category,
        message.kind()
    );
    if show_data && message.kind() == BlazeMessageKind::Message {
        if let Ok(text) = message.data_text() {
            println!("text: {}", truncate(&text, 256));
        }
    }
}

fn truncate(value: &str, limit: usize) -> String {
    let mut chars = value.chars();
    let truncated: String = chars.by_ref().take(limit).collect();
    if chars.next().is_none() {
        return value.to_string();
    }
    format!("{truncated}...")
}

fn parse_args() -> Result<Args, Error> {
    let mut keystore = None;
    let mut timeout_seconds = 30;
    let mut ack = false;
    let mut max_reconnects = None;
    let mut show_data = false;

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
            "--max-reconnects" => {
                let value = iter.next().ok_or_else(|| {
                    Error::Input("--max-reconnects requires a number".to_string())
                })?;
                max_reconnects =
                    Some(value.parse().map_err(|_| {
                        Error::Input("--max-reconnects must be a number".to_string())
                    })?);
            }
            "--show-data" => {
                show_data = true;
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
        max_reconnects,
        show_data,
    })
}

fn print_usage() {
    eprintln!(
        "usage: cargo run --example blaze_loop -- --keystore <path> [--timeout-seconds 30] [--ack] [--max-reconnects 3] [--show-data]"
    );
}
