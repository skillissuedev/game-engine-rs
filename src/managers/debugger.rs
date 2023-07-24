use colored::Colorize;
use crate::framework::DebugMode;

pub static mut DEBUG: DebugMode = DebugMode::None;

#[derive(Debug)]
pub enum Error {
    FileLoadingError
}

pub fn crash(text: &str) {
    println!("{}", "project baldej crashed!".red());
    println!("{}\n{}", "Error:".red(), text.red());
    println!("\n\n\n\n");
    panic!(); 
}

pub fn error(text: &str) {
    println!("{}\n{}", "Error:".red(), text.red());
    println!("\n");
}

pub fn warn(text: &str) {
    println!("{}\n{}", "Warning:".yellow(), text.yellow());
    println!("\n");
}

pub fn print_if_debug(text: &str) {
    unsafe {
        match DEBUG {
            DebugMode::Full => println!("{}", text),
            _ => ()
        }
    }
}
