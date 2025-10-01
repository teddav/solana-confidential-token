use std::sync::Arc;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;

mod confidential;
mod testing;
mod token;
mod token_metadata;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Arc::new(RpcClient::new_with_commitment(
        String::from("http://localhost:8899"),
        CommitmentConfig::confirmed(),
    ));

    // testing::main().await?;

    // token::create_mint(&client).await?;
    // token::get_mint(&client).await?;

    // token_metadata::create_mint(&client).await?;

    confidential::main(client.clone()).await?;

    Ok(())
}
