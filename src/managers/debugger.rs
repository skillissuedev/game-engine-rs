use colored::Colorize;

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
