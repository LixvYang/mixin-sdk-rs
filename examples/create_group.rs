use mixin_sdk_rs::conversation::{create_group_conversation, Participant};
use mixin_sdk_rs::safe::SafeUser;

#[tokio::main]
async fn main() -> Result<(), mixin_sdk_rs::error::Error> {
    let user = SafeUser::new_from_env()?;
    let participants_raw = std::env::var("PARTICIPANT_IDS")
        .map_err(|_| mixin_sdk_rs::error::Error::Input("PARTICIPANT_IDS is not set".to_string()))?;

    let participants: Vec<Participant> = participants_raw
        .split(',')
        .map(|id| id.trim())
        .filter(|id| !id.is_empty())
        .map(|id| Participant {
            user_id: id.to_string(),
            role: None,
            created_at: None,
        })
        .collect();

    if participants.is_empty() {
        return Err(mixin_sdk_rs::error::Error::Input("no participants provided".to_string()));
    }

    let name = std::env::var("GROUP_NAME").unwrap_or_else(|_| "Rust SDK Group".to_string());
    let announcement = std::env::var("GROUP_ANNOUNCEMENT").unwrap_or_else(|_| "Hello".to_string());

    let conversation = create_group_conversation(&name, &announcement, participants, &user).await?;
    println!("created group {}", conversation.conversation_id);
    Ok(())
}
