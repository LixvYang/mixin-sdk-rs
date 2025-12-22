use mixin_sdk_rs::address::{AddressInput, create_address};
use mixin_sdk_rs::safe::SafeUser;

#[tokio::main]
async fn main() -> Result<(), mixin_sdk_rs::error::Error> {
    let user = SafeUser::new_from_env()?;
    let asset_id = std::env::var("ASSET_ID")
        .map_err(|_| mixin_sdk_rs::error::Error::Input("ASSET_ID is not set".to_string()))?;
    let destination = std::env::var("DESTINATION")
        .map_err(|_| mixin_sdk_rs::error::Error::Input("DESTINATION is not set".to_string()))?;
    let label = std::env::var("ADDRESS_LABEL").unwrap_or_else(|_| "Rust SDK".to_string());
    let tag = std::env::var("ADDRESS_TAG").unwrap_or_default();

    let input = AddressInput {
        asset_id: &asset_id,
        label: &label,
        destination: &destination,
        tag: &tag,
    };

    let address = create_address(&input, &user).await?;
    println!("address id: {}", address.address_id);
    Ok(())
}
