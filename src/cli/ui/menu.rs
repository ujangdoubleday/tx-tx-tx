use colored::Colorize;
use inquire::Select;
use super::handlers::{self, clear_screen};

#[derive(Clone, Copy)]
pub enum MainMenuItem {
    Signature,
    Quit,
}

impl std::fmt::Display for MainMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MainMenuItem::Signature => write!(f, "1. Signature"),
            MainMenuItem::Quit => write!(f, "2. Quit"),
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

pub fn run() -> anyhow::Result<()> {
    loop {
        clear_screen();
        print_banner();

        let options = vec![MainMenuItem::Signature, MainMenuItem::Quit];

        let selected = Select::new("Choose an option:", options)
            .with_page_size(2)
            .prompt();

        match selected {
            Ok(MainMenuItem::Signature) => {
                signature_menu()?;
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
