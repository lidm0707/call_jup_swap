use anchor_client::{
    Client, Cluster,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        instruction::Instruction,
        pubkey::Pubkey,
        signature::{Signer, read_keypair_file},
        transaction::Transaction,
    },
};
use anyhow::Result;
use spl_associated_token_account::{get_associated_token_address, instruction as ata_instruction};
use spl_token::instruction as token_instruction;
use std::{env, path::PathBuf, rc::Rc, str::FromStr};

fn main() -> Result<()> {
    // --- Load keypair ---
    let home = env::var("HOME")?;
    let keypair_path = PathBuf::from(home).join(".config/solana/id.json");
    let payer = Rc::new(
        read_keypair_file(&keypair_path)
            .map_err(|e| anyhow::anyhow!("failed to read keypair: {}", e))?,
    );

    // --- Client ---
    let url = "http://localhost:8899"; // Mainnet RPC endpoint
    let client = Client::new_with_options(
        Cluster::Custom(url.to_string(), url.to_string()),
        payer.clone(),
        CommitmentConfig::confirmed(),
    );
    let program = client.program(Pubkey::default())?;
    let rpc = program.rpc();

    // --- USDC Mint (Mainnet) ---
    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;

    // --- Recipient ---
    let recipient = Pubkey::from_str("CNdJxMoD8L8C6RxydLakcgEjQb5nUTsi1p3JyEKEmsZC")?;

    // --- Find ATA (associated token account) ---
    let sender_ata = get_associated_token_address(&payer.pubkey(), &usdc_mint);
    let recipient_ata = get_associated_token_address(&recipient, &usdc_mint);

    println!("Sender ATA: {}", sender_ata);
    println!("Recipient ATA: {}", recipient_ata);

    // --- Check and create ATAs if they don't exist ---
    let mut instructions: Vec<Instruction> = vec![];

    // Check if sender ATA exists
    if rpc.get_account(&sender_ata).is_err() {
        println!("Sender ATA does not exist, creating...");
        let create_sender_ata_ix = ata_instruction::create_associated_token_account(
            &payer.pubkey(), // Payer (funds the account creation)
            &payer.pubkey(), // Owner of the ATA
            &usdc_mint,      // Mint
            &spl_token::ID,  // Token program
        );
        instructions.push(create_sender_ata_ix);
    }

    // Check if recipient ATA exists
    if rpc.get_account(&recipient_ata).is_err() {
        println!("Recipient ATA does not exist, creating...");
        let create_recipient_ata_ix = ata_instruction::create_associated_token_account(
            &payer.pubkey(), // Payer (funds the account creation)
            &recipient,      // Owner of the ATA
            &usdc_mint,      // Mint
            &spl_token::ID,  // Token program
        );
        instructions.push(create_recipient_ata_ix);
    }

    // --- Transfer 1 USDC (6 decimals) ---
    let amount: u64 = 1_000_000; // 1.000000 USDC
    let transfer_ix = token_instruction::transfer(
        &spl_token::ID,  // Token program
        &sender_ata,     // From
        &recipient_ata,  // To
        &payer.pubkey(), // Authority
        &[],             // Signer seeds
        amount,
    )?;
    instructions.push(transfer_ix);

    // --- Build & send transaction ---
    let bh = rpc.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &[payer.as_ref()],
        bh,
    );

    let sig = rpc.send_and_confirm_transaction(&tx)?;
    println!("✅ USDC transfer success, sig = {}", sig);

    Ok(())
}
