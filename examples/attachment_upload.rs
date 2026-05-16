use mixin_sdk_rs::attachment::{create_attachment, upload_attachment_file};
use mixin_sdk_rs::error::Error;
use mixin_sdk_rs::safe::SafeUser;

#[derive(Debug, Default)]
struct Args {
    keystore: String,
    file: Option<String>,
    create_only: bool,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = parse_args()?;
    let user = SafeUser::new_from_file(&args.keystore)?;

    if args.create_only {
        let attachment = create_attachment(&user).await?;
        println!("attachment_id: {}", attachment.attachment_id);
        if let Some(upload_url) = attachment.upload_url {
            println!("upload_url: {}", upload_url);
        }
        if let Some(view_url) = attachment.view_url {
            println!("view_url: {}", view_url);
        }
        return Ok(());
    }

    let file = args.file.ok_or_else(|| {
        Error::Input("--file is required unless --create-only is passed".to_string())
    })?;
    let uploaded = upload_attachment_file(file, &user).await?;
    println!("attachment_id: {}", uploaded.attachment_id);
    if let Some(view_url) = uploaded.view_url {
        println!("view_url: {}", view_url);
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
            "--file" => {
                args.file = Some(
                    iter.next()
                        .ok_or_else(|| Error::Input("--file requires a path".to_string()))?,
                );
            }
            "--create-only" => {
                args.create_only = true;
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => return Err(Error::Input(format!("unknown argument: {other}"))),
        }
    }

    if args.keystore.is_empty() {
        print_usage();
        return Err(Error::Input("--keystore is required".to_string()));
    }
    if !args.create_only && args.file.is_none() {
        print_usage();
        return Err(Error::Input(
            "--file is required unless --create-only is passed".to_string(),
        ));
    }

    Ok(args)
}

fn print_usage() {
    eprintln!(
        "usage: cargo run --example attachment_upload -- --keystore <path> (--file <path> | --create-only)"
    );
}
