use mixin_sdk_rs::safe::SafeUser;
use mixin_sdk_rs::safe::register_safe_user;

#[tokio::main]
async fn main() -> Result<(), mixin_sdk_rs::error::Error> {
    let user = SafeUser::new_from_env()?;
    let registered = register_safe_user(&user).await?;
    println!("registered user_id: {}", registered.user_id);
    Ok(())
}
