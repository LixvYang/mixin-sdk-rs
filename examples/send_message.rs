use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use mixin_sdk_rs::message::{MessageRequest, post_message};
use mixin_sdk_rs::safe::SafeUser;
use mixin_sdk_rs::utils::unique_conversation_id;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), mixin_sdk_rs::error::Error> {
    let user = SafeUser::new_from_env()?;
    let recipient_id = std::env::var("RECIPIENT_ID")
        .map_err(|_| mixin_sdk_rs::error::Error::Input("RECIPIENT_ID is not set".to_string()))?;

    let conversation_id = unique_conversation_id(&user.user_id, &recipient_id);
    let data = STANDARD.encode("hello from rust sdk");

    let message = MessageRequest {
        conversation_id,
        recipient_id: Some(recipient_id),
        message_id: Uuid::new_v4().to_string(),
        category: "PLAIN_TEXT".to_string(),
        data_base64: data,
        representative_id: None,
        quote_message_id: None,
    };

    post_message(message, &user).await?;
    println!("message sent");
    Ok(())
}
