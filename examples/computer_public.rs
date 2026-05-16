use mixin_sdk_rs::computer::{get_computer_deployed_assets, get_computer_info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let info = get_computer_info().await?;
    println!("observer: {}", info.observer);
    println!("payer: {}", info.payer);
    println!("height: {}", info.height);
    println!(
        "mtg: app={} members={} threshold={}",
        info.members.app_id,
        info.members.members.len(),
        info.members.threshold
    );
    println!(
        "operation: asset={} price={}",
        info.params.operation.asset, info.params.operation.price
    );

    let assets = get_computer_deployed_assets().await?;
    println!("deployed_assets: {}", assets.len());
    for asset in assets.iter().take(5) {
        println!(
            "{} {} chain={} address={} solana_asset_id={}",
            asset.symbol,
            asset.name,
            asset.chain_id,
            asset.address,
            asset.solana_asset_id()
        );
    }

    Ok(())
}
