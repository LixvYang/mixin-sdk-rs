//! examples/get_me.rs
//!
//! # run example
//!
//! 1. create a `keystore.json` file, content as follows:
//!
//! ```json
//! {
//!   "app_id": "YOUR_USER_ID",
//!   "session_id": "YOUR_SESSION_ID",
//!   "session_private_key": "YOUR_PRIVATE_KEY",
//!   "server_public_key": "YOUR_SERVER_PUBLIC_KEY",
//!   "spend_private_key": "YOUR_SPEND_PRIVATE_KEY"
//! }
//! ```
//!
//! 2. set the environment variable `TEST_KEYSTORE_PATH` to the path of your `keystore.json` file:
//!
//! ```bash
//! export TEST_KEYSTORE_PATH="/path/to/your/keystore.json"
//!
//! cargo run --example get_me --all-features
//! ```

use mixin_sdk_rs::error::Error;
use mixin_sdk_rs::safe::SafeUser;
use mixin_sdk_rs::user;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let user = SafeUser::new_from_env()?;
    let me = user::request_user_me(&user).await?;
    println!("me: \n{:#?}", me);

    Ok(())
}
