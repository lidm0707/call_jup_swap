use anchor_client::{
    Client, Cluster,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{Signer, read_keypair_file},
        system_instruction::transfer, // ✅ import ตรงนี้
        transaction::Transaction,
    },
};
use anyhow::Result;
use std::{env, path::PathBuf, rc::Rc, str::FromStr};

fn main() -> Result<()> {
    // --- Load keypair ---
    let home = env::var("HOME")?;
    let keypair_path = PathBuf::from(home).join(".config/solana/id.json");
    let payer = Rc::new(
        read_keypair_file(keypair_path)
            .map_err(|e| anyhow::anyhow!("failed to read keypair: {}", e))?,
    );

    // --- Client ---
    let url = "http://localhost:8899"; // หรือ https://api.devnet.solana.com
    let client = Client::new_with_options(
        Cluster::Custom(url.to_string(), url.to_string()),
        payer.clone(),
        CommitmentConfig::confirmed(),
    );
    let program = client.program(Pubkey::default())?;
    let rpc = program.rpc();

    // --- Recipient ---
    let recipient = Pubkey::from_str("CNdJxMoD8L8C6RxydLakcgEjQb5nUTsi1p3JyEKEmsZC")?;

    // --- Build transfer instruction ---
    let lamports = 1_000_000; // 0.001 SOL
    let ix = transfer(&payer.pubkey(), &recipient, lamports);

    // --- Build & send tx ---
    let bh = rpc.get_latest_blockhash()?;
    let tx =
        Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[payer.as_ref()], bh);
    let sig = rpc.send_and_confirm_transaction(&tx)?;

    println!("✅ SOL transfer success, sig = {}", sig);
    Ok(())
}
