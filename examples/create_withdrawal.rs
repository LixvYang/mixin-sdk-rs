use mixin_sdk_rs::safe::SafeUser;
use mixin_sdk_rs::withdrawal::create_withdrawal;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), mixin_sdk_rs::error::Error> {
    let user = SafeUser::new_from_env()?;
    let address_id = std::env::var("ADDRESS_ID")
        .map_err(|_| mixin_sdk_rs::error::Error::Input("ADDRESS_ID is not set".to_string()))?;
    let amount = std::env::var("AMOUNT")
        .map_err(|_| mixin_sdk_rs::error::Error::Input("AMOUNT is not set".to_string()))?;
    let fee = std::env::var("FEE")
        .map_err(|_| mixin_sdk_rs::error::Error::Input("FEE is not set".to_string()))?;
    let memo = std::env::var("MEMO").ok();

    let trace_id = std::env::var("TRACE_ID").unwrap_or_else(|_| Uuid::new_v4().to_string());
    let withdrawal = create_withdrawal(
        &address_id,
        &amount,
        &fee,
        &trace_id,
        memo.as_deref(),
        &user,
    )
    .await?;
    println!(
        "withdrawal id: {}",
        withdrawal.withdrawal_id.unwrap_or_default()
    );
    Ok(())
}
