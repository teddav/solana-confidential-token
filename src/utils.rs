use anyhow::Result;
use std::time::Duration;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, signature::Keypair, signer::Signer,
    transaction::Transaction,
};

pub async fn airdrop(client: &RpcClient, account: &Keypair, amount: u64) -> Result<()> {
    let transaction_signature = client
        .request_airdrop(&account.pubkey(), amount * LAMPORTS_PER_SOL)
        .await?;
    loop {
        println!("Airdropping...");
        tokio::time::sleep(Duration::from_secs(1)).await;
        if client.confirm_transaction(&transaction_signature).await? {
            return Ok(());
        }
    }
}

async fn transfer(
    rpc_client: RpcClient,
    sender: &Keypair,
    recipient: &Pubkey,
    amount: u64,
) -> Result<()> {
    println!("Funding account {} with {} lamports...", recipient, amount);

    let fund_signature = rpc_client
        .send_and_confirm_transaction(&Transaction::new_signed_with_payer(
            &[solana_sdk::system_instruction::transfer(
                &sender.pubkey(), // From
                recipient,        // To
                amount,           // Amount in lamports
            )],
            Some(&sender.pubkey()),
            &[&sender],
            rpc_client.get_latest_blockhash().await?,
        ))
        .await?;

    println!("Fund Transaction Signature: {}", fund_signature);

    Ok(())
}
