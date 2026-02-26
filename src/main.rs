use std::io::{self, Write};

fn main() {
    println!("diskparted version 0.1.0");
    println!("Type 'exit' to quit.\n");

    let mut input = String::new();

    loop {
        print!("diskparted> ");
        io::stdout().flush().expect("Failed to flush stdout");

        input.clear();

        if io::stdin().read_line(&mut input).is_err() {
            println!("Error reading input.");
            continue;
        }

        let command = input.trim();

        match command {
            "" => continue,

            "exit" | "quit" => {
                println!("Exiting diskparted.");
                break;
            }

            _ => {
                println!("Unknown command: {}", command);
            }
        }
    }
}