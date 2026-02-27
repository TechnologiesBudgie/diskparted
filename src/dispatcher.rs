/*
 * DiskParted - A Disk Management Tool
 * Copyright (C) 2026 DiskParted Team
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
use crate::commands::{clean, list, select, select_partition, create, format, help, delete, rescan, shrink};
use crate::context::Context;

/// Dispatch a user command to the correct module
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
        }
        "create" => create::run(&parts[1..], ctx),
        "delete" => delete::run(&parts[1..], ctx),
        "rescan" => rescan::run(&parts[1..], ctx),
        "shrink" => shrink::run(&parts[1..], ctx),
        "help" => help::run(),
        "format" => format::run(&parts[1..], ctx),
        _ => println!("Unknown command: {}", parts[0]),
    }
}
