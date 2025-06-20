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
- [Error Handling](#error-handling)
- [License](#license)

## Features

*   **Complete**: Supports most APIs for Mixin Network and Mixin Messenger.
*   **Secure**: All API requests are automatically signed with JWT.
*   **Idiomatic Rust**: Designed to be asynchronous from the ground up using `tokio`.
*   **Developer Friendly**: Provides clear error handling and a simple, function-based API.

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

Now you can load the `SafeUser` from your keystore and make API calls.

```rust
use mixin_sdk_rs::safe::SafeUser;
use mixin_sdk_rs::user;
use mixin_sdk_rs::error::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // 1. Set the environment variable to point to your keystore file.
    std::env::set_var("TEST_KEYSTORE_PATH", "/path/to/your/keystore.json");

    // 2. Load the user credentials from the keystore.
    let user = SafeUser::new_from_env()?;

    // 3. Call the API.
    println!("Fetching user profile...");
    let me = user::request_user_me(&user).await?;

    println!("Success! User ID: {}", me.user_id);
    if let Some(name) = me.full_name {
        println!("Full Name: {}", name);
    }

    Ok(())
}
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
        mixin_sdk_rs::error::Error::Reqwest(e) => {
            // Error from the underlying HTTP client (e.g., network issues)
            eprintln!("[Network Error] {}", e);
        }
        mixin_sdk_rs::error::Error::Serde(e) => {
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

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
