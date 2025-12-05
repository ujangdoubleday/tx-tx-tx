use colored::Colorize;
use std::io::{self, Write};

pub const WIDTH: usize = 80;

pub fn clear_screen() {
    print!("\x1B[2J\x1B[3J\x1B[1;1H");
    std::io::Write::flush(&mut std::io::stdout()).ok();
}

pub fn print_separator() {
    println!("{}", "─".repeat(WIDTH));
}

pub fn print_line(label: &str, value: &str, color_fn: fn(&str) -> colored::ColoredString) {
    let label_width = 10;
    println!("  {:<width$} {}", format!("{}:", label).bold(), color_fn(value), width = label_width);
}

pub fn read_input_line(prompt: &str) -> anyhow::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}
