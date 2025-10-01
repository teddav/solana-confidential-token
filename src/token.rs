use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    program_pack::Pack, pubkey::Pubkey, signature::Keypair, signer::Signer,
    system_instruction::create_account, transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address_with_program_id,
    instruction::create_associated_token_account_idempotent,
};
use spl_token_2022::{
    ID as TOKEN_2022_PROGRAM_ID,
    extension::{BaseStateWithExtensions, StateWithExtensions},
    instruction::{initialize_mint2, mint_to_checked, transfer_checked},
    state::Mint,
};

use crate::utils::airdrop;

pub async fn get_mint(client: &RpcClient, mint_address: &Pubkey) -> anyhow::Result<()> {
    // use solana_sdk::pubkey;
    // let mint_address = pubkey!("H1QK6xB68sod5sHppHRSsCF83Zu849xzCteF6aLPww75");
    let mint_data = client.get_account_data(&mint_address).await?;
    let mint = StateWithExtensions::<Mint>::unpack(&mint_data).unwrap();
    let extension_types = mint.get_extension_types().unwrap();

    println!("{:#?}", mint);
    println!("{:#?}", extension_types);

    Ok(())
}

pub async fn create_associated_token_account(
    client: &RpcClient,
    owner: &Keypair,
    mint: &Keypair,
) -> anyhow::Result<Pubkey> {
    println!("\nCreating associated token account");
    let associated_token_address = get_associated_token_address_with_program_id(
        &owner.pubkey(),
        &mint.pubkey(),
        &TOKEN_2022_PROGRAM_ID,
    );
    println!("Associated token address: {}", associated_token_address);

    let create_ata_ix = create_associated_token_account_idempotent(
        &owner.pubkey(), // payer
        &owner.pubkey(),
        &mint.pubkey(),
        &TOKEN_2022_PROGRAM_ID,
    );

    let mut transaction = Transaction::new_with_payer(&[create_ata_ix], Some(&owner.pubkey()));
    transaction.sign(&[&owner], client.get_latest_blockhash().await?);

    println!("Sending transaction to create associated token account");
    match client.send_and_confirm_transaction(&transaction).await {
        Ok(signature) => println!("Transaction Signature: {}", signature),
        Err(err) => eprintln!("Error sending transaction: {}", err),
    }
    Ok(associated_token_address)
}

async fn mint_to(
    client: &RpcClient,
    authority: &Keypair,
    mint: &Keypair,
    associated_token_address: &Pubkey,
    amount: u64,
) -> anyhow::Result<()> {
    println!("\nMinting to associated token account");

    let mint_decimals = client
        .get_token_account_balance(&associated_token_address)
        .await?
        .decimals;
    println!("Mint decimals: {}", mint_decimals);
    let amount_to_mint = amount * 10_u64.pow(mint_decimals as u32);
    println!("Amount to mint: {}", amount_to_mint);

    let mint_to_ix = mint_to_checked(
        &TOKEN_2022_PROGRAM_ID,
        &mint.pubkey(),
        &associated_token_address,
        &authority.pubkey(),
        &[&authority.pubkey()],
        amount_to_mint,
        mint_decimals,
    )?;

    let mut transaction = Transaction::new_with_payer(&[mint_to_ix], Some(&authority.pubkey()));

    transaction.sign(&[&authority], client.get_latest_blockhash().await?);

    println!("Sending transaction to mint to associated token account");
    match client.send_and_confirm_transaction(&transaction).await {
        Ok(signature) => println!("Transaction Signature: {}", signature),
        Err(err) => eprintln!("Error sending transaction: {}", err),
    }

    Ok(())
}

pub async fn transfer_to(
    client: &RpcClient,
    authority: &Keypair,
    mint: &Keypair,
    sender_token_account: &Pubkey,
    amount: u64,
) -> anyhow::Result<()> {
    println!("\nTransferring to another account");

    let recipient = Keypair::new();
    println!("Recipient: {}", recipient.pubkey());
    airdrop(client, &recipient, 3).await?;

    let recipient_ata = create_associated_token_account(client, &recipient, mint).await?;
    println!("Recipient ATA: {}", recipient_ata);

    let decimals = client
        .get_token_account_balance(&sender_token_account)
        .await?
        .decimals;
    let transfer_amount = amount * 10_u64.pow(decimals as u32);

    let transfer_ix = transfer_checked(
        &TOKEN_2022_PROGRAM_ID,
        &sender_token_account,
        &mint.pubkey(),
        &recipient_ata,
        &authority.pubkey(),
        &[&authority.pubkey()],
        transfer_amount,
        decimals,
    )?;
    let mut transaction = Transaction::new_with_payer(&[transfer_ix], Some(&authority.pubkey()));

    transaction.sign(&[&authority], client.get_latest_blockhash().await?);

    match client.send_and_confirm_transaction(&transaction).await {
        Ok(signature) => println!("Transaction Signature: {}", signature),
        Err(err) => eprintln!("Error transferring tokens: {}", err),
    }

    Ok(())
}

pub async fn create_mint(client: &RpcClient) -> anyhow::Result<()> {
    println!("Start creating mint");

    let authority_keypair = Keypair::new();
    println!("Authority keypair: {}", authority_keypair.pubkey());
    let mint_account = Keypair::new();
    println!("Mint account: {}", mint_account.pubkey());

    let mint_account_len = Mint::LEN;
    let mint_account_rent = client
        .get_minimum_balance_for_rent_exemption(mint_account_len)
        .await?;

    let create_mint_account_ix = create_account(
        &authority_keypair.pubkey(),
        &mint_account.pubkey(),
        mint_account_rent,
        mint_account_len as u64,
        &TOKEN_2022_PROGRAM_ID,
    );

    let initialize_mint_ix = initialize_mint2(
        &TOKEN_2022_PROGRAM_ID,
        &mint_account.pubkey(),
        &authority_keypair.pubkey(),
        Some(&authority_keypair.pubkey()),
        9,
    )?;

    airdrop(client, &authority_keypair, 5).await?;

    let mut transaction = Transaction::new_with_payer(
        &[create_mint_account_ix, initialize_mint_ix],
        Some(&authority_keypair.pubkey()),
    );

    transaction.sign(
        &[&authority_keypair, &mint_account],
        client.get_latest_blockhash().await?,
    );

    match client.send_and_confirm_transaction(&transaction).await {
        Ok(signature) => println!("Transaction Signature: {}", signature),
        Err(err) => eprintln!("Error sending transaction: {}", err),
    }

    let associated_token_address =
        create_associated_token_account(client, &authority_keypair, &mint_account).await?;
    mint_to(
        client,
        &authority_keypair,
        &mint_account,
        &associated_token_address,
        100,
    )
    .await?;

    transfer_to(
        client,
        &authority_keypair,
        &mint_account,
        &associated_token_address,
        10,
    )
    .await?;

    Ok(())
}
