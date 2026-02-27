use crate::commands::{clean, list, select, create, format};
use crate::context::Context;

pub fn dispatch(input: &str, ctx: &mut Context) {
    let parts: Vec<&str> = input.split_whitespace().collect();

    if parts.is_empty() {
        return;
    }

    match parts[0].to_lowercase().as_str() {
        "clean" => clean::run(&parts[1..], ctx),
        "list" => list::run(&parts[1..], ctx),
        "select" => select::run(&parts[1..], ctx),
        "create" => create::run(&parts[1..], ctx),
        "help" => crate::commands::help::run(),
	"format" => format::run(&parts[1..], ctx),
        _ => println!("Unknown command: {}", parts[0]),
    }
}
