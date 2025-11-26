use colored::Colorize;
use inquire::Select;
use super::handlers::{self, clear_screen};

#[derive(Clone, Copy)]
pub enum MainMenuItem {
    Signatures,
    Transactions,
    Quit,
}

impl std::fmt::Display for MainMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MainMenuItem::Signatures => write!(f, "1. Signatures"),
            MainMenuItem::Transactions => write!(f, "2. Transactions"),
            MainMenuItem::Quit => write!(f, "3. Quit"),
        }
    }
}

#[derive(Clone, Copy)]
pub enum SignatureMenuItem {
    SignMessage,
    VerifyMessage,
    Back,
    Quit,
}

#[derive(Clone, Copy)]
pub enum TransactionMenuItem {
    Transfer,
    Back,
    Quit,
}

#[derive(Clone, Copy)]
pub enum NetworkMenuItem {
    EthereumMainnet,
    EthereumSepolia,
    Back,
    Quit,
}

impl std::fmt::Display for SignatureMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignatureMenuItem::SignMessage => write!(f, "1. Sign Message"),
            SignatureMenuItem::VerifyMessage => write!(f, "2. Verify Message"),
            SignatureMenuItem::Back => write!(f, "3. Back"),
            SignatureMenuItem::Quit => write!(f, "4. Quit"),
        }
    }
}

impl std::fmt::Display for TransactionMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionMenuItem::Transfer => write!(f, "1. Transfer"),
            TransactionMenuItem::Back => write!(f, "2. Back"),
            TransactionMenuItem::Quit => write!(f, "3. Quit"),
        }
    }
}

impl std::fmt::Display for NetworkMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkMenuItem::EthereumMainnet => write!(f, "1. Ethereum Mainnet"),
            NetworkMenuItem::EthereumSepolia => write!(f, "2. Ethereum Sepolia Testnet"),
            NetworkMenuItem::Back => write!(f, "3. Back"),
            NetworkMenuItem::Quit => write!(f, "4. Quit"),
        }
    }
}

fn print_banner() {
    println!("{}", r#"
 /$$$$$$$$ /$$   /$$     /$$$$$$$$ /$$   /$$     /$$$$$$$$ /$$   /$$
|__  $$__/| $$  / $$    |__  $$__/| $$  / $$    |__  $$__/| $$  / $$
   | $$   |  $$/ $$/       | $$   |  $$/ $$/       | $$   |  $$/ $$/
   | $$    \  $$$$/ /$$$$$$| $$    \  $$$$/ /$$$$$$| $$    \  $$$$/ 
   | $$     >$$  $$|______/| $$     >$$  $$|______/| $$     >$$  $$ 
   | $$    /$$/\  $$       | $$    /$$/\  $$       | $$    /$$/\  $$
   | $$   | $$  \ $$       | $$   | $$  \ $$       | $$   | $$  \ $$
   |__/   |__/  |__/       |__/   |__/  |__/       |__/   |__/  |__/
                                                                
 by ujangdoubleday
"#.bright_blue().bold());
}



fn signature_menu() -> anyhow::Result<()> {
    loop {
        clear_screen();
        print_banner();

        let options = vec![
            SignatureMenuItem::SignMessage,
            SignatureMenuItem::VerifyMessage,
            SignatureMenuItem::Back,
            SignatureMenuItem::Quit,
        ];

        let selected = Select::new("Choose an option:", options)
            .with_page_size(4)
            .prompt();

        match selected {
            Ok(SignatureMenuItem::SignMessage) => {
                if let Err(e) = handlers::handle_sign() {
                    println!("{}", format!("❌ {}", e).red().bold());
                }
                println!();
                println!("Press Enter to continue...");
                std::io::stdin().read_line(&mut String::new())?;
            }
            Ok(SignatureMenuItem::VerifyMessage) => {
                if let Err(e) = handlers::handle_verify() {
                    println!("{}", format!("❌ {}", e).red().bold());
                }
                println!();
                println!("Press Enter to continue...");
                std::io::stdin().read_line(&mut String::new())?;
            }
            Ok(SignatureMenuItem::Back) => {
                return Ok(());
            }
            Ok(SignatureMenuItem::Quit) => {
                clear_screen();
                println!("{}", "👋 Goodbye!".green().bold());
                std::process::exit(0);
            }
            Err(_) => {
                clear_screen();
                println!("{}", "👋 Goodbye!".green().bold());
                std::process::exit(0);
            }
        }
    }
}

fn transaction_menu() -> anyhow::Result<()> {
    loop {
        clear_screen();
        print_banner();

        let options = vec![
            TransactionMenuItem::Transfer,
            TransactionMenuItem::Back,
            TransactionMenuItem::Quit,
        ];

        let selected = Select::new("Choose an option:", options)
            .with_page_size(3)
            .prompt();

        match selected {
            Ok(TransactionMenuItem::Transfer) => {
                transfer_menu()?;
            }
            Ok(TransactionMenuItem::Back) => {
                return Ok(());
            }
            Ok(TransactionMenuItem::Quit) => {
                clear_screen();
                println!("{}", "👋 Goodbye!".green().bold());
                std::process::exit(0);
            }
            Err(_) => {
                clear_screen();
                println!("{}", "👋 Goodbye!".green().bold());
                std::process::exit(0);
            }
        }
    }
}

fn transfer_menu() -> anyhow::Result<()> {
    loop {
        clear_screen();
        print_banner();

        let options = vec![
            NetworkMenuItem::EthereumMainnet,
            NetworkMenuItem::EthereumSepolia,
            NetworkMenuItem::Back,
            NetworkMenuItem::Quit,
        ];

        let selected = Select::new("Choose a network:", options)
            .with_page_size(4)
            .prompt();

        match selected {
            Ok(NetworkMenuItem::EthereumMainnet) => {
                // For now, only Sepolia is implemented for development
                println!("{}", "🚧 Ethereum Mainnet transfer not yet implemented. Use Sepolia for development.".yellow().bold());
                println!();
                println!("Press Enter to continue...");
                std::io::stdin().read_line(&mut String::new())?;
            }
            Ok(NetworkMenuItem::EthereumSepolia) => {
                if let Err(e) = handlers::handle_transfer_sepolia() {
                    println!("{}", format!("❌ {}", e).red().bold());
                }
                println!();
                println!("Press Enter to continue...");
                std::io::stdin().read_line(&mut String::new())?;
            }
            Ok(NetworkMenuItem::Back) => {
                return Ok(());
            }
            Ok(NetworkMenuItem::Quit) => {
                clear_screen();
                println!("{}", "👋 Goodbye!".green().bold());
                std::process::exit(0);
            }
            Err(_) => {
                clear_screen();
                println!("{}", "👋 Goodbye!".green().bold());
                std::process::exit(0);
            }
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    loop {
        clear_screen();
        print_banner();

        let options = vec![MainMenuItem::Signatures, MainMenuItem::Transactions, MainMenuItem::Quit];

        let selected = Select::new("Choose an option:", options)
            .with_page_size(2)
            .prompt();

        match selected {
            Ok(MainMenuItem::Signatures) => {
                signature_menu()?;
            }
            Ok(MainMenuItem::Transactions) => {
                transaction_menu()?;
            }
            Ok(MainMenuItem::Quit) => {
                clear_screen();
                println!("{}", "👋 Goodbye!".green().bold());
                break;
            }
            Err(_) => {
                clear_screen();
                println!("{}", "👋 Goodbye!".green().bold());
                break;
            }
        }
    }
    Ok(())
}
