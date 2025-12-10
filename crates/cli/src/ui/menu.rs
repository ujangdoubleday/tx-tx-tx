use colored::Colorize;
use inquire::Select;
use super::handlers::{self, clear_screen};
use x_core as core;

#[derive(Clone, Copy)]
pub enum MainMenuItem {
    TheGate,
    Signatures,
    TransferEth,
    Compile,
    Quit,
}

#[derive(Clone, Debug)]
pub enum NetworkMenuChoice {
    SelectNetwork(String),
    Back,
    Quit,
}

#[derive(Clone)]
pub struct NetworkMenuOption {
    pub choice: NetworkMenuChoice,
    pub display: String,
}

impl std::fmt::Display for NetworkMenuOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display)
    }
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



#[derive(Clone, Copy)]
pub enum GateFeatureMenuItem {
    Deploy,
    SmartContractInvoker,
    Back,
    Quit,
}

impl std::fmt::Display for GateFeatureMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateFeatureMenuItem::Deploy => write!(f, "1. Deploy Smart Contract"),
            GateFeatureMenuItem::SmartContractInvoker => write!(f, "2. Smart Contract Invoker"),
            GateFeatureMenuItem::Back => write!(f, "3. Back"),
            GateFeatureMenuItem::Quit => write!(f, "4. Quit"),
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

        let networks = core::networks::load_networks()?;
        let mut options = Vec::new();
        
        for (index, network) in networks.iter().enumerate() {
            options.push(NetworkMenuOption {
                choice: NetworkMenuChoice::SelectNetwork(network.id.clone()),
                display: format!("{}. {}", index + 1, network.name),
            });
        }
        
        options.push(NetworkMenuOption {
            choice: NetworkMenuChoice::Back,
            display: format!("{}. Back", options.len() + 1),
        });
        options.push(NetworkMenuOption {
            choice: NetworkMenuChoice::Quit,
            display: format!("{}. Quit", options.len() + 1),
        });

        let selected = Select::new("Choose a network:", options)
            .with_page_size(10)
            .prompt();

        match selected {
            Ok(option) => {
                match option.choice {
                    NetworkMenuChoice::SelectNetwork(network_id) => {
                        if network_id == "testnet_sepolia" {
                            if let Err(e) = handlers::handle_transfer_sepolia() {
                                println!("{}", format!("❌ {}", e).red().bold());
                            }
                        } else {
                            println!("{}", "🚧 Transfer not yet implemented for this network.".yellow().bold());
                        }
                        println!();
                        println!("Press Enter to continue...");
                        std::io::stdin().read_line(&mut String::new())?;
                    }
                    NetworkMenuChoice::Back => {
                        return Ok(());
                    }
                    NetworkMenuChoice::Quit => {
                        clear_screen();
                        println!("{}", "👋 Goodbye!".green().bold());
                        std::process::exit(0);
                    }
                }
            }
            Err(_) => {
                clear_screen();
                println!("{}", "👋 Goodbye!".green().bold());
                std::process::exit(0);
            }
        }
    }
}

fn gate_feature_menu(network_id: &str) -> anyhow::Result<()> {
    loop {
        clear_screen();
        print_banner();

        let options = vec![
            GateFeatureMenuItem::Deploy,
            GateFeatureMenuItem::SmartContractInvoker,
            GateFeatureMenuItem::Back,
            GateFeatureMenuItem::Quit,
        ];

        let selected = Select::new("Choose a feature:", options)
            .with_page_size(4)
            .prompt();

        match selected {
            Ok(GateFeatureMenuItem::Deploy) => {
                match handlers::handle_gate_deploy(network_id) {
                    Ok(_) => {
                        println!();
                        println!("Press Enter to continue...");
                        std::io::stdin().read_line(&mut String::new())?;
                    }
                    Err(e) => {
                        let err_msg = e.to_string();
                        if err_msg != "__BACK__" {
                            println!("{}", format!("❌ {}", e).red().bold());
                            println!();
                            println!("Press Enter to continue...");
                            std::io::stdin().read_line(&mut String::new())?;
                        }
                    }
                }
            }
            Ok(GateFeatureMenuItem::SmartContractInvoker) => {
                match handlers::handle_smart_contract_invoker(network_id) {
                    Ok(_) => {
                        println!();
                        println!("Press Enter to continue...");
                        std::io::stdin().read_line(&mut String::new())?;
                    }
                    Err(e) => {
                        let err_msg = e.to_string();
                        if err_msg != "__BACK__" {
                            println!("{}", format!("❌ {}", e).red().bold());
                            println!();
                            println!("Press Enter to continue...");
                            std::io::stdin().read_line(&mut String::new())?;
                        }
                    }
                }
            }
            Ok(GateFeatureMenuItem::Back) => {
                return Ok(());
            }
            Ok(GateFeatureMenuItem::Quit) => {
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

        let networks = core::networks::load_networks()?;
        let mut options = Vec::new();
        
        for (index, network) in networks.iter().enumerate() {
            options.push(NetworkMenuOption {
                choice: NetworkMenuChoice::SelectNetwork(network.id.clone()),
                display: format!("{}. {}", index + 1, network.name),
            });
        }
        
        options.push(NetworkMenuOption {
            choice: NetworkMenuChoice::Back,
            display: format!("{}. Back", options.len() + 1),
        });
        options.push(NetworkMenuOption {
            choice: NetworkMenuChoice::Quit,
            display: format!("{}. Quit", options.len() + 1),
        });

        let selected = Select::new("Choose a network for The Gate:", options)
            .with_page_size(10)
            .prompt();

        match selected {
            Ok(option) => {
                match option.choice {
                    NetworkMenuChoice::SelectNetwork(network_id) => {
                        gate_feature_menu(&network_id)?;
                    }
                    NetworkMenuChoice::Back => {
                        return Ok(());
                    }
                    NetworkMenuChoice::Quit => {
                        clear_screen();
                        println!("{}", "👋 Goodbye!".green().bold());
                        std::process::exit(0);
                    }
                }
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
