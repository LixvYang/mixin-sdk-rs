use mixin_sdk_rs::output::list_unspent_outputs;
use mixin_sdk_rs::safe::SafeUser;
use mixin_sdk_rs::utils::hash_members;

#[tokio::main]
async fn main() -> Result<(), mixin_sdk_rs::error::Error> {
    let user = SafeUser::new_from_env()?;
    let members_hash = hash_members([user.user_id.as_str()]);
    let outputs = list_unspent_outputs(&members_hash, 1, None, &user).await?;
    println!("outputs: {}", outputs.len());
    Ok(())
}
