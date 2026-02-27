use crate::commands::{clean, list, select, create, format, help, select_partition};
use crate::context::Context;

pub fn dispatch(input: &str, ctx: &mut Context) {
    let parts: Vec<&str> = input.split_whitespace().collect();

    if parts.is_empty() {
        return;
    }

    match parts[0].to_lowercase().as_str() {
        "clean" => clean::run(&parts[1..], ctx),
        "list" => list::run(&parts[1..], ctx),
        "select" => {
            if parts.len() > 1 && parts[1].to_lowercase() == "partition" {
                select_partition::run(&parts[1..], ctx);
            } else {
                select::run(&parts[1..], ctx);
            }
        },
        "create" => create::run(&parts[1..], ctx),
        "help" => help::run(),
        "format" => format::run(&parts[1..], ctx),
        _ => println!("Unknown command: {}", parts[0]),
    }
}
