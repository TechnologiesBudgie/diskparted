/*
 * DiskParted - A Disk Management Tool
 * Copyright (C) 2026 DiskParted Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
mod context;
mod dispatcher;
mod utils;
mod commands;

use context::Context;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

fn main() {
    println!("diskparted version 0.1.1");
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
