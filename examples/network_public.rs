use mixin_sdk_rs::error::Error;
use mixin_sdk_rs::{chain, fiats, network};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let info = network::read_network_info().await?;
    println!("assets_count: {:?}", info.assets_count);
    println!("snapshots_count: {:?}", info.snapshots_count);
    println!("assets_in_info: {}", info.assets.len());
    println!("chains_in_info: {}", info.chains.len());

    let chains = chain::read_network_chains().await?;
    println!("chains: {}", chains.len());

    let top_assets = network::read_network_assets_top(Some("ALL")).await?;
    println!("top_assets: {}", top_assets.len());
    if let Some(asset) = top_assets.first() {
        println!("top_asset: {} {:?}", asset.asset_id, asset.symbol);
    }

    let fiat_rates = fiats::get_fiats().await?;
    println!("fiats: {}", fiat_rates.len());
    if let Some(usd) = fiat_rates.iter().find(|fiat| fiat.code == "USD") {
        println!("USD_rate: {}", usd.rate);
    }

    Ok(())
}
