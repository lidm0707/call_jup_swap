use anchor_client::{
    Client, Cluster,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{Signer, read_keypair_file},
        transaction::VersionedTransaction,
    },
};
use anyhow::Result;
use jup_ag::{QuoteConfig, SwapRequest};
use std::{env, path::PathBuf, rc::Rc, str::FromStr};

fn main() -> Result<()> {
    // --- Load keypair from $HOME ---
    let home = env::var("HOME")?;
    let keypair_path = PathBuf::from(home).join(".config/solana/id.json");
    let payer = Rc::new(
        read_keypair_file(keypair_path)
            .map_err(|e| anyhow::anyhow!("failed to read keypair: {}", e))?,
    );

    // --- Anchor client (Surfpool RPC) ---
    let url = "http://localhost:8899";
    let client = Client::new_with_options(
        Cluster::Custom(url.to_string(), url.to_string()),
        payer.clone(),
        CommitmentConfig::confirmed(),
    );

    let dummy_program_id = Pubkey::from_str("11111111111111111111111111111111")?;
    let program = client.program(dummy_program_id)?;
    let rpc = program.rpc();

    // --- Mints ---
    let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;
    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;

    // --- สร้าง runtime ชั่วคราว สำหรับ jup_ag ---
    let rt = tokio::runtime::Runtime::new()?;

    // === 1. Request quote ===
    let quote = rt.block_on(jup_ag::quote(
        sol_mint,
        usdc_mint,
        1_000_000_000, // 1 SOL
        QuoteConfig {
            slippage_bps: Some(50),
            ..Default::default()
        },
    ))?;
    println!("Best quote out_amount = {}", quote.out_amount);

    // === 2. Build SwapRequest ===
    let swap_req = SwapRequest::new(payer.pubkey(), quote);

    // === 3. Get swap transaction ===
    let swap = rt.block_on(jup_ag::swap(swap_req))?;
    let vtx: VersionedTransaction = swap.swap_transaction;

    // --- 4. Re-sign transaction ---
    let recent_blockhash = rpc.get_latest_blockhash()?;
    let mut msg = vtx.message.clone();
    msg.set_recent_blockhash(recent_blockhash);

    let vtx_signed = VersionedTransaction::try_new(msg, &[payer.as_ref()])?;

    // === 5. ส่ง transaction ===
    let sig = rpc.send_and_confirm_transaction(&vtx_signed)?;
    println!("✅ Swap success, signature: {}", sig);

    Ok(())
}
