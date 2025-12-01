use colored::Colorize;
use inquire::Select;
use super::handlers::{self, clear_screen};

#[derive(Clone, Copy)]
pub enum MainMenuItem {
    TheGate,
    Signatures,
    TransferEth,
    Compile,
    Quit,
}

impl std::fmt::Display for MainMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MainMenuItem::TheGate => write!(f, "1. The Gate"),
            MainMenuItem::Signatures => write!(f, "2. Signatures"),
            MainMenuItem::TransferEth => write!(f, "3. Transfer ETH"),
            MainMenuItem::Compile => write!(f, "4. Compile Smart Contracts"),
            MainMenuItem::Quit => write!(f, "5. Quit"),
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
pub enum NetworkMenuItem {
    EthereumMainnet,
    EthereumSepolia,
    Back,
    Quit,
}

#[derive(Clone, Copy)]
pub enum GateNetworkMenuItem {
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

impl std::fmt::Display for GateNetworkMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateNetworkMenuItem::EthereumMainnet => write!(f, "1. Ethereum Mainnet"),
            GateNetworkMenuItem::EthereumSepolia => write!(f, "2. Ethereum Sepolia Testnet"),
            GateNetworkMenuItem::Back => write!(f, "3. Back"),
            GateNetworkMenuItem::Quit => write!(f, "4. Quit"),
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



fn network_menu() -> anyhow::Result<()> {
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

fn gate_menu() -> anyhow::Result<()> {
    loop {
        clear_screen();
        print_banner();

        let options = vec![
            GateNetworkMenuItem::EthereumMainnet,
            GateNetworkMenuItem::EthereumSepolia,
            GateNetworkMenuItem::Back,
            GateNetworkMenuItem::Quit,
        ];

        let selected = Select::new("Choose a network for The Gate:", options)
            .with_page_size(4)
            .prompt();

        match selected {
            Ok(GateNetworkMenuItem::EthereumMainnet) => {
                if let Err(e) = handlers::handle_gate_mainnet() {
                    println!("{}", format!("❌ {}", e).red().bold());
                }
                println!();
                println!("Press Enter to continue...");
                std::io::stdin().read_line(&mut String::new())?;
            }
            Ok(GateNetworkMenuItem::EthereumSepolia) => {
                if let Err(e) = handlers::handle_gate_sepolia() {
                    println!("{}", format!("❌ {}", e).red().bold());
                }
                println!();
                println!("Press Enter to continue...");
                std::io::stdin().read_line(&mut String::new())?;
            }
            Ok(GateNetworkMenuItem::Back) => {
                return Ok(());
            }
            Ok(GateNetworkMenuItem::Quit) => {
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

        let options = vec![MainMenuItem::TheGate, MainMenuItem::Signatures, MainMenuItem::TransferEth, MainMenuItem::Compile, MainMenuItem::Quit];

        let selected = Select::new("Choose an option:", options)
            .with_page_size(5)
            .prompt();

        match selected {
            Ok(MainMenuItem::TheGate) => {
                gate_menu()?;
            }
            Ok(MainMenuItem::Signatures) => {
                signature_menu()?;
            }
            Ok(MainMenuItem::TransferEth) => {
                network_menu()?;
            }
            Ok(MainMenuItem::Compile) => {
                if let Err(e) = handlers::handle_compile_smart_contracts() {
                    println!("{}", format!("❌ {}", e).red().bold());
                }
                println!();
                println!("Press Enter to continue...");
                std::io::stdin().read_line(&mut String::new())?;
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
