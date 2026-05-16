<div align="center">

<h1>Mixin SDK for Rust</h1>

**A complete, secure, and idiomatic Rust SDK for the Mixin Network & Mixin Messenger.**

<br />

[![CI Status](https://img.shields.io/github/actions/workflow/status/lixvyang/mixin-sdk-rs/ci.yml?branch=master&style=flat-square)](https://github.com/lixvyang/mixin-sdk-rs/actions)
[![Crates.io](https://img.shields.io/crates/v/mixin-sdk-rs.svg?style=flat-square)](https://crates.io/crates/mixin-sdk-rs)

</div>

---

## Table of Contents

- [Table of Contents](#table-of-contents)
- [Features](#features)
- [Installation](#installation)
- [Getting Started](#getting-started)
  - [Step 1: Create your Keystore File](#step-1-create-your-keystore-file)
  - [Step 2: Write Your Code](#step-2-write-your-code)
- [Running Examples](#running-examples)
- [Examples Index](#examples-index)
- [Error Handling](#error-handling)
- [License](#license)

## Features

*   **Complete**: Supports most APIs for Mixin Network and Mixin Messenger, including Safe, messaging, Blaze, OAuth, apps, users, PIN, payments, circles, collectibles, deposits, public network reads, and legacy transfer/multisig compatibility.
*   **Safe Ready**: Supports MIX addresses, offline MIN invoices, and Safe v5 raw transaction build/encode/decode/sign flows.
*   **Secure**: All API requests are automatically signed with JWT.
*   **Idiomatic Rust**: Designed to be asynchronous from the ground up using `tokio`.
*   **Developer Friendly**: Provides clear error handling, a `Client` facade, and function-based module APIs.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
mixin-sdk-rs = { git = "https://github.com/lixvyang/mixin-sdk-rs" }
tokio = { version = "1", features = ["full"] }
```

## Getting Started

Follow these two simple steps to start using the SDK.

### Step 1: Create your Keystore File

It is highly recommended to manage your bot's credentials using a `keystore.json` file instead of hardcoding them. Create a file named `keystore.json` with the following structure:

```json
{
  "app_id": "YOUR_USER_ID",
  "session_id": "YOUR_SESSION_ID",
  "session_private_key": "YOUR_PRIVATE_KEY",
  "server_public_key": "YOUR_SERVER_PUBLIC_KEY",
  "spend_private_key": "YOUR_SPEND_PRIVATE_KEY"
}
```
> **Security Note**: Make sure to add this file to your `.gitignore` to prevent committing your secrets to version control.

### Step 2: Write Your Code

Now you can load a `Client` from your keystore and make API calls.

```rust
use mixin_sdk_rs::Client;
use mixin_sdk_rs::error::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // 1. Set the environment variable to point to your keystore file.
    std::env::set_var("TEST_KEYSTORE_PATH", "/path/to/your/keystore.json");

    // 2. Load the client from the keystore.
    let client = Client::from_env()?;

    // 3. Call the API.
    println!("Fetching user profile...");
    let me = client.me().await?;

    println!("Success! User ID: {}", me.user_id);
    if let Some(name) = me.full_name {
        println!("Full Name: {}", name);
    }

    Ok(())
}
```

The lower-level module functions are still available when you want the Go-style
`function(..., &safe_user)` shape:

```rust
let safe_user = client.safe_user();
let me = mixin_sdk_rs::user::request_user_me(safe_user).await?;
```

## Running Examples

The `/examples` directory contains various usage examples. You can run any example using `cargo run`.

For instance, to run the `get_me.rs` example:

1.  Make sure you have created your `keystore.json` file.
2.  Set the environment variable.
    ```bash
    export TEST_KEYSTORE_PATH="/path/to/your/keystore.json"
    ```
3.  Run the example.
    ```bash
    cargo run --example get_me --all-features
    ```

## Examples Index

All examples expect `TEST_KEYSTORE_PATH` unless noted otherwise.

- `get_me`: Fetch `/safe/me`
- `register_safe_user`: Register Safe user with spend key (requires a fresh user)
- `send_message`: Send a plain text message with `--keystore`, `--recipient`, and optional `--text`
- `encrypted_message_check`: Fetch recipient sessions and build a Go-compatible encrypted payload for custom decrypting clients; normal Messenger clients will show it as unreadable text
- `create_group`: Create a group conversation (requires `PARTICIPANT_IDS`, optional `GROUP_NAME`/`GROUP_ANNOUNCEMENT`)
- `list_outputs`: List unspent outputs
- `create_address`: Create a withdrawal address (requires `ASSET_ID`, `DESTINATION`, optional `ADDRESS_LABEL`/`ADDRESS_TAG`)
- `create_withdrawal`: Create a withdrawal (requires `ADDRESS_ID`, `AMOUNT`, `FEE`, optional `MEMO`/`TRACE_ID`)
- `attachment_upload`: Create an attachment upload slot or upload a local file with `--keystore` and `--file`
- `blaze_listen_once`: Connect to Blaze, list pending messages, and wait for one message
- `blaze_loop`: Run the high-level Blaze listener loop with reconnect and optional timeout
- `network_public`: Fetch public network info, chains, top assets, and fiat rates
- `computer_public`: Fetch public Mixin Computer info and deployed assets without reading a keystore
- `sdk_surface`: Print request payload shapes for OAuth, apps, users, payments, circles, collectibles, deposits, external proxy, Safe snapshots, inscription extras, Mixin Computer extras, and legacy transfer/multisig without reading a keystore
- `safe_online_check`: Load a keystore from `--keystore`, fetch `/safe/me`, and list Safe unspent outputs
- `safe_transfer_verify`: Build a Safe v5 transfer, verify it with the sequencer, sign locally, and only submit when `--send` is passed
- `safe_transaction_status`: Fetch a Safe transaction request by request id
- `safe_multisig_status`: Fetch a Safe multisig request by request id or transaction hash
- `safe_multisig_sign`: Fetch a Safe multisig request, sign it with this keystore's spend key, and submit the signer raw

Example commands:

```bash
export TEST_KEYSTORE_PATH="/path/to/keystore.json"
```

```bash
cargo run --example get_me --all-features
cargo run --example register_safe_user --all-features
```

```bash
cargo run --example send_message -- \
  --keystore /path/to/keystore.json \
  --recipient <target-user-id> \
  --text "hello from rust sdk"
cargo run --example encrypted_message_check -- \
  --keystore /path/to/keystore.json \
  --recipient <target-user-id> \
  --text "hello from rust sdk encrypted message"
# This mirrors Go EncryptMessageData and creates an encrypted binary envelope.
# Only send when the recipient client explicitly decrypts this SDK envelope:
# cargo run --example encrypted_message_check -- ... --send --recipient-decrypts
```

```bash
cargo run --example attachment_upload -- \
  --keystore /path/to/keystore.json \
  --file /path/to/file.bin
cargo run --example attachment_upload -- \
  --keystore /path/to/keystore.json \
  --create-only
```

```bash
cargo run --example blaze_listen_once -- \
  --keystore /path/to/keystore.json \
  --connect-only
cargo run --example blaze_listen_once -- \
  --keystore /path/to/keystore.json \
  --timeout-seconds 10
cargo run --example blaze_loop -- \
  --keystore /path/to/keystore.json \
  --timeout-seconds 30
# add --ack to acknowledge messages, and --show-data to print decoded text payloads
cargo run --example network_public
cargo run --example computer_public
cargo run --example sdk_surface
```

```bash
export PARTICIPANT_IDS="user-id-1,user-id-2"
export GROUP_NAME="Rust SDK Group"
export GROUP_ANNOUNCEMENT="Hello"
cargo run --example create_group --all-features
```

```bash
cargo run --example list_outputs --all-features
```

```bash
cargo run --example safe_online_check -- --keystore /path/to/keystore.json
cargo run --example safe_transfer_verify -- \
  --keystore /path/to/keystore.json \
  --asset <asset-id-or-kernel-hash> \
  --receiver <uuid-or-MIX-or-XIN-or-app-creator> \
  --amount 0.001 \
  --extra-text "verify only"
cargo run --example safe_transaction_status -- \
  --keystore /path/to/keystore.json \
  --request-id <trace-id>
cargo run --example safe_multisig_status -- \
  --keystore /path/to/keystore.json \
  --id <request-id-or-hash>
cargo run --example safe_multisig_sign -- \
  --keystore /path/to/signer-keystore.json \
  --id <request-id-or-hash>
```

```bash
export ASSET_ID="asset-id"
export DESTINATION="destination"
export ADDRESS_LABEL="Rust SDK"
export ADDRESS_TAG=""
cargo run --example create_address --all-features
```

```bash
export ADDRESS_ID="address-id"
export AMOUNT="1"
export FEE="0.001"
export MEMO="memo"
export TRACE_ID="trace-id"
cargo run --example create_withdrawal --all-features
```

## Error Handling

All API functions return a `Result<T, mixin_sdk_rs::error::Error>`. You can match on the `Error` enum to handle different failure scenarios.

```rust
// ... inside an async function
if let Err(err) = user::request_user_me(&user).await {
    match err {
        mixin_sdk_rs::error::Error::Api(e) => {
            // Error returned by the Mixin API
            eprintln!("[API Error] Code: {}, Description: {}", e.code, e.description);
            if e.code == 401 {
                eprintln!("=> Unauthorized. Please check your credentials.");
            }
        }
        mixin_sdk_rs::error::Error::Request(e) => {
            // Error from the underlying HTTP client (e.g., network issues)
            eprintln!("[Network Error] {}", e);
        }
        mixin_sdk_rs::error::Error::Json(e) => {
            // Error during JSON serialization/deserialization
            eprintln!("[Serialization Error] {}", e);
        }
        _ => {
            // Other kinds of errors
            eprintln!("[An unexpected error occurred] {}", err);
        }
    }
}
```

## License

This project is licensed under the Apache-2.0 License. See the [LICENSE](LICENSE) file for details.
