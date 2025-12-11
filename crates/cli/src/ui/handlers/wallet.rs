use colored::Colorize;
use inquire::Text;
use x_wallet::WalletGenerator;

pub fn handle_generate_wallet() -> anyhow::Result<()> {
    println!("{}", "\n🔑 Wallet Generator".cyan().bold());
    println!("{}", "─".repeat(50).cyan());

    let count_input = Text::new("Enter number of wallets to generate (default: 1):")
        .prompt()?;

    let count_str = if count_input.trim().is_empty() {
        "1".to_string()
    } else {
        count_input
    };

    let count: usize = count_str.trim().parse()
        .map_err(|_| anyhow::anyhow!("Invalid number. Please enter a valid integer."))?;

    if count == 0 {
        return Err(anyhow::anyhow!("Number of wallets must be greater than 0"));
    }

    let filename_input = Text::new("Enter filename (default: wallets):")
        .prompt()?;

    let filename_base = if filename_input.trim().is_empty() {
        "wallets".to_string()
    } else {
        filename_input
            .trim()
            .to_lowercase()
            .replace(" ", "_")
            .trim_end_matches(".json")
            .to_string()
    };

    let filename = format!("{}.json", filename_base);
    let output_path = format!("wallet/{}", filename);

    println!("\n{}Generating {} wallet(s)...", "⏳ ".blue(), count);

    let wallets = WalletGenerator::generate_and_save(count, &output_path)?;

    println!("{}", format!("✓ {} wallet(s) generated successfully!", count).green().bold());
    println!("{}", format!("Saved to: {}", output_path).green());
    println!();

    for (i, wallet) in wallets.iter().enumerate() {
        println!("{} [Wallet {}]", "📌".cyan(), i + 1);
        println!("   ID:      {}", wallet.id);
        println!("   Address: {}", wallet.address);
        println!("   Key:     {}", wallet.privatekey);
        println!();
    }

    Ok(())
}
