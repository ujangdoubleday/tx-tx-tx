use x_wallet;

pub async fn handle_gen_wallet(count: usize, filename: String) -> anyhow::Result<()> {
    println!("Generating {} wallet(s)...", count);
    let output_path = format!("wallet/{}", filename);
    let wallets = x_wallet::WalletGenerator::generate_and_save(count, &output_path)?;
    println!("✓ {} wallet(s) generated and saved to {}", count, output_path);
    for (i, wallet) in wallets.iter().enumerate() {
        println!("  [{}] ID: {}", i + 1, wallet.id);
        println!("      Address: {}", wallet.address);
    }
    Ok(())
}
