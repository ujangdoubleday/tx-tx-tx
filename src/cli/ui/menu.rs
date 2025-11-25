use colored::Colorize;
use inquire::Select;
use super::handlers::{self, clear_screen};

#[derive(Clone, Copy)]
pub enum MenuItem {
    Sign,
    Verify,
    Exit,
}

impl std::fmt::Display for MenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MenuItem::Sign => write!(f, "✍️  Sign Message"),
            MenuItem::Verify => write!(f, "✓ Verify Message"),
            MenuItem::Exit => write!(f, "❌ Exit"),
        }
    }
}

fn print_header() {
    println!("\n{}", "🔐 EVM MESSAGE SIGNING & VERIFICATION TOOL 🔐".bright_blue().bold());
}

pub fn run() -> anyhow::Result<()> {
    loop {
        clear_screen();
        print_header();

        let options = vec![MenuItem::Sign, MenuItem::Verify, MenuItem::Exit];

        let selected = Select::new("Choose an option:", options)
            .with_page_size(3)
            .prompt();

        match selected {
            Ok(MenuItem::Sign) => {
                if let Err(e) = handlers::handle_sign() {
                    println!("{}", format!("❌ {}", e).red().bold());
                }
            }
            Ok(MenuItem::Verify) => {
                if let Err(e) = handlers::handle_verify() {
                    println!("{}", format!("❌ {}", e).red().bold());
                }
            }
            Ok(MenuItem::Exit) => {
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
