pub mod cli;
pub mod config;
pub mod process;

use std::io::Write;
pub fn confirm(message: &str) -> bool {
    print!("{} [y/N] ", message);
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_lowercase() == "y"
}
