mod context;
mod dispatcher;
mod utils;
mod commands;

use context::Context;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

fn main() {
    println!("diskparted version 0.1.0");
    println!("Type 'exit' to quit.\n");

    let mut rl = DefaultEditor::new().unwrap();
    let mut ctx = Context::default();

    loop {
        let readline = rl.readline("DISKPARTED> ");

        match readline {
            Ok(line) => {
                let command = line.trim();

                if command.is_empty() {
                    continue;
                }

                rl.add_history_entry(command).unwrap();

                if command.eq_ignore_ascii_case("exit") || command.eq_ignore_ascii_case("quit") {
                    println!("Exiting diskparted.");
                    break;
                }

                // ✅ Make sure dispatch is public in dispatcher.rs
                dispatcher::dispatch(command, &mut ctx);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
