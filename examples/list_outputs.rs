use mixin_sdk_rs::output::list_unspent_outputs;
use mixin_sdk_rs::safe::SafeUser;

#[tokio::main]
async fn main() -> Result<(), mixin_sdk_rs::error::Error> {
    let user = SafeUser::new_from_env()?;
    let outputs = list_unspent_outputs(&user.user_id, 1, None, &user).await?;
    println!("outputs: {}", outputs.len());
    Ok(())
}
