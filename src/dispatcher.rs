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

use crate::commands::{
    // DiskPart-compatible
    active, add, assign, automount, break_cmd, clean, convert, create, delete,
    detail, extend, filesystems, format, gpt, help, import_cmd, inactive, list,
    offline, online, recover, rem, remove, repair, rescan, retain, san, select,
    setid, shrink, uniqueid,
    // Unimplementable stubs
    impossible,
    // Virtual disk manager
    vdisk,
    // Linux extensions
    benchmark, encrypt, smart, snapshot, wipe,
};
use crate::context::Context;

/// Dispatch a user command string to the correct module.
pub fn dispatch(input: &str, ctx: &mut Context) {
    let parts: Vec<&str> = input.split_whitespace().collect();

    if parts.is_empty() {
        return;
    }

    match parts[0].to_lowercase().as_str() {
        // ── DiskPart-compatible commands ───────────────────────────────────
        "active"      => active::run(&parts[1..], ctx),
        "add"         => add::run(&parts[1..], ctx),
        "assign"      => assign::run(&parts[1..], ctx),
        "automount"   => automount::run(&parts[1..], ctx),
        "break"       => break_cmd::run(&parts[1..], ctx),
        "clean"       => clean::run(&parts[1..], ctx),
        "convert"     => convert::run(&parts[1..], ctx),
        "create"      => create::run(&parts[1..], ctx),
        "delete"      => delete::run(&parts[1..], ctx),
        "detail"      => detail::run(&parts[1..], ctx),
        "extend"      => extend::run(&parts[1..], ctx),
        "filesystems" => filesystems::run(&parts[1..], ctx),
        "format"      => format::run(&parts[1..], ctx),
        "gpt"         => gpt::run(&parts[1..], ctx),
        "help"        => help::run(),
        "import"      => import_cmd::run(&parts[1..], ctx),
        "inactive"    => inactive::run(&parts[1..], ctx),
        "list"        => list::run(&parts[1..], ctx),
        "offline"     => offline::run(&parts[1..], ctx),
        "online"      => online::run(&parts[1..], ctx),
        "recover"     => recover::run(&parts[1..], ctx),
        "rem"         => rem::run(&parts[1..], ctx),
        "remove"      => remove::run(&parts[1..], ctx),
        "repair"      => repair::run(&parts[1..], ctx),
        "rescan"      => rescan::run(&parts[1..], ctx),
        "retain"      => retain::run(&parts[1..], ctx),
        "san"         => san::run(&parts[1..], ctx),
        "select"      => select::run(&parts[1..], ctx),
        "set"         => dispatch_set(&parts[1..], ctx),
        "shrink"      => shrink::run(&parts[1..], ctx),
        "uniqueid"    => uniqueid::run(&parts[1..], ctx),
        "vdisk"       => vdisk::run(&parts[1..], ctx),

        // ── Unimplementable Windows-only stubs ─────────────────────────────
        "attach"      => impossible::attach(&parts[1..], ctx),
        "attributes"  => impossible::attributes(&parts[1..], ctx),
        "compact"     => impossible::compact(&parts[1..], ctx),
        "detach"      => impossible::detach(&parts[1..], ctx),
        "expand"      => impossible::expand(&parts[1..], ctx),
        "merge"       => impossible::merge(&parts[1..], ctx),

        // ── Linux extensions ───────────────────────────────────────────────
        "benchmark"   => benchmark::run(&parts[1..], ctx),
        "encrypt"     => encrypt::run(&parts[1..], ctx),
        "smart"       => smart::run(&parts[1..], ctx),
        "snapshot"    => snapshot::run(&parts[1..], ctx),
        "wipe"        => wipe::run(&parts[1..], ctx),

        _ => println!(
            "Unknown command: '{}'. Type 'help' for a list of commands.",
            parts[0]
        ),
    }
}

/// `set` is a multi-subcommand: currently only `set id=...` is supported.
fn dispatch_set(args: &[&str], ctx: &mut Context) {
    if args.is_empty() {
        println!("Usage: set id={{<hex_byte>|<GUID>|<alias>}} [override] [noerr]");
        return;
    }
    setid::run(args, ctx);
}
