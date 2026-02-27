use crate::commands::{clean, help, list, select};
use crate::context::Context;

pub fn dispatch(input: &str, ctx: &mut Context) {
    let parts: Vec<&str> = input.split_whitespace().collect();

    if parts.is_empty() {
        return;
    }

    match parts[0].to_lowercase().as_str() {
        "list" => list::run(&parts[1..], ctx),
        "select" => select::run(&parts[1..], ctx),
        "clean" => clean::run(&parts[1..], ctx),
        "help" => help::run(),
        "?" => help::run(),
        _ => println!("Unknown command. Type 'help' for available commands."),
    }
}