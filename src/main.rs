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

/// Returns true if the current effective user ID is 0 (root).
fn is_root() -> bool {
    std::process::Command::new("id")
        .arg("-u")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "0")
        .unwrap_or(false)
}

fn main() {
    // DiskParted requires root privileges for all disk operations.
    // If not running as root, re-exec the current binary under sudo.
    if !is_root() {
        eprintln!("DiskParted requires root privileges. Re-launching with sudo...");

        let exe = std::env::current_exe()
            .expect("Failed to determine current executable path");

        let status = std::process::Command::new("sudo")
            .arg(exe)
            .args(std::env::args().skip(1))
            .status()
            .expect("Failed to execute sudo. Is sudo installed?");

        std::process::exit(status.code().unwrap_or(1));
    }

    println!("diskparted version 1.0.1");
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
