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

//! Shared utility functions used across command modules.

use std::io::{self, Write};

/// Prompt the user with a yes/no question and return `true` if they answer `y` or `yes`.
///
/// Prints `<prompt> [y/N]: ` and reads a line from stdin.
/// Defaults to **no** on empty input or any non-yes answer.
///
/// # Example
/// ```
/// if !utils::confirm("This will erase all data. Continue?") {
///     println!("Aborted.");
///     return;
/// }
/// ```
pub fn confirm(prompt: &str) -> bool {
    print!("{} [y/N]: ", prompt);
    io::stdout().flush().unwrap_or(());

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }

    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}
