use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey, signature::Keypair, signer::Signer, system_instruction::create_account,
    transaction::Transaction,
};
use spl_token_2022::{
    ID as TOKEN_2022_PROGRAM_ID,
    extension::{BaseStateWithExtensions, ExtensionType, StateWithExtensions, metadata_pointer},
    instruction::initialize_mint2,
    state::Mint,
};
use spl_token_metadata_interface::{
    instruction::initialize as metadata_initialize, state::TokenMetadata,
};
use spl_type_length_value::variable_len_pack::VariableLenPack;

use crate::utils::airdrop;

pub async fn get_mint(client: &RpcClient, mint_address: &Pubkey) -> anyhow::Result<()> {
    let mint_data = client.get_account_data(&mint_address).await?;
    let mint = StateWithExtensions::<Mint>::unpack(&mint_data).unwrap();
    let extension_types = mint.get_extension_types().unwrap();

    println!("{:?}", mint);
    println!("{:?}", extension_types);

    Ok(())
}

pub async fn create_mint(client: &RpcClient) -> anyhow::Result<()> {
    println!("Start creating mint");

    let authority_keypair = Keypair::new();
    println!("Authority keypair: {}", authority_keypair.pubkey());
    let mint_account = Keypair::new();
    println!("Mint account: {}", mint_account.pubkey());

    let metadata = TokenMetadata {
        update_authority: Some(authority_keypair.pubkey()).try_into()?,
        mint: mint_account.pubkey(),
        name: "DAVDAV".to_string(),
        symbol: "DAV".to_string(),
        uri: "https://raw.githubusercontent.com/solana-developers/opos-asset/main/assets/DeveloperPortal/metadata.json".to_string(),
        additional_metadata: vec![],
    };
    let packed_len = metadata.get_packed_len()?;
    println!("packed_len {packed_len}");

    // Size of MetadataExtension 2 bytes for type, 2 bytes for length
    let metadata_extension = 2 + 2;

    let extensions = [ExtensionType::MetadataPointer];
    let mint_account_len = ExtensionType::try_calculate_account_len::<Mint>(&extensions)?;
    let mint_account_rent = client
        .get_minimum_balance_for_rent_exemption(mint_account_len + packed_len + metadata_extension)
        .await?;
    println!("mint_account_rent {mint_account_rent}");

    let create_mint_account_ix = create_account(
        &authority_keypair.pubkey(),
        &mint_account.pubkey(),
        mint_account_rent,
        mint_account_len as u64,
        &TOKEN_2022_PROGRAM_ID,
    );

    let metadata_pointer_initialize_ix = metadata_pointer::instruction::initialize(
        &TOKEN_2022_PROGRAM_ID,
        &mint_account.pubkey(),
        Some(authority_keypair.pubkey()),
        Some(mint_account.pubkey()),
    )?;

    let initialize_mint_ix = initialize_mint2(
        &TOKEN_2022_PROGRAM_ID,
        &mint_account.pubkey(),
        &authority_keypair.pubkey(),
        Some(&authority_keypair.pubkey()),
        9,
    )?;

    let metadata_initialize_ix = metadata_initialize(
        &TOKEN_2022_PROGRAM_ID,
        &metadata.mint,
        &metadata.update_authority.0,
        &metadata.mint,
        &authority_keypair.pubkey(),
        metadata.name,
        metadata.symbol,
        metadata.uri,
    );

    airdrop(client, &authority_keypair, 5).await?;

    let mut transaction = Transaction::new_with_payer(
        &[
            create_mint_account_ix,
            metadata_pointer_initialize_ix,
            initialize_mint_ix,
            metadata_initialize_ix,
        ],
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

    // get_mint(client, &mint_account.pubkey()).await?;

    Ok(())
}
