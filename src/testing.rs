use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, native_token::LAMPORTS_PER_SOL, pubkey::Pubkey,
    signature::Signer, signer::keypair::Keypair, system_instruction, transaction::Transaction,
};
use std::str::FromStr;

pub async fn main() -> Result<()> {
    println!("Testing");

    // transfer().await?;
    // transaction().await?;
    pda().await?;

    Ok(())
}

pub async fn transfer() -> Result<()> {
    // Generate sender and recipient keypairs
    let sender = Keypair::new();
    let recipient = Keypair::new();

    // Define the amount to transfer
    let transfer_amount = LAMPORTS_PER_SOL / 100; // 0.01 SOL

    // Create a transfer instruction for transferring SOL from sender to recipient
    let transfer_instruction =
        system_instruction::transfer(&sender.pubkey(), &recipient.pubkey(), transfer_amount);

    println!("{:#?}", transfer_instruction);

    Ok(())
}

pub async fn transaction() -> Result<()> {
    let connection = RpcClient::new_with_commitment(
        "http://localhost:8899".to_string(),
        CommitmentConfig::confirmed(),
    );

    // Fetch the latest blockhash and last valid block height
    let blockhash = connection.get_latest_blockhash().await?;

    // Generate sender and recipient keypairs
    let sender = Keypair::new();
    let recipient = Keypair::new();

    // Create a transfer instruction for transferring SOL from sender to recipient
    let transfer_instruction = system_instruction::transfer(
        &sender.pubkey(),
        &recipient.pubkey(),
        LAMPORTS_PER_SOL / 100, // 0.01 SOL
    );

    let mut transaction =
        Transaction::new_with_payer(&[transfer_instruction], Some(&sender.pubkey()));
    transaction.sign(&[&sender], blockhash);

    println!("{:#?}", transaction);
    Ok(())
}

async fn pda() -> Result<()> {
    let program_address = Pubkey::from_str("11111111111111111111111111111111")?;
    let optional_seed_bytes = b"helloWorld";
    let optional_seed_address = Pubkey::from_str("B9Lf9z5BfNPT4d5KMeaBFx8x1G4CULZYR1jA2kmxRDka")?;
    let seeds: &[&[u8]] = &[optional_seed_bytes, optional_seed_address.as_ref()];
    let (pda, bump) = Pubkey::find_program_address(seeds, &program_address);

    println!("PDA: {}", pda);
    println!("Bump: {}", bump);

    // Loop through all bump seeds (255 down to 0)
    for bump in (0..=255).rev() {
        let seeds_and_bump = [optional_seed_bytes, optional_seed_address.as_ref(), &[bump]];
        match Pubkey::create_program_address(&seeds_and_bump, &program_address) {
            Ok(pda) => println!("bump {}: {}", bump, pda),
            Err(err) => println!("bump {}: {}", bump, err),
        }
    }

    Ok(())
}
