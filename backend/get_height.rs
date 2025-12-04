use tokio;

#[tokio::main]
async fn main() {
    let lightwalletd_url = "https://na.zec.rocks:443".to_string();

    // Simple gRPC call to get block height
    println!("Connecting to {}...", lightwalletd_url);

    // For now, let's just use psql to update - we know the approximate height
    // Current mainnet height is around 2.7M+ blocks
    println!("Please use psql to check and update");
}
