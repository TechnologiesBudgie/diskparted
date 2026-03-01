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
use crate::context::Context;

/// SAN — display or set the SAN (Storage Area Network) policy.
///
/// DiskPart syntax:
///   san [policy={ onlineall | offlineall | offlineshared }]
///
/// The SAN policy controls whether newly discovered disks are automatically
/// brought online and whether they are writable. This is a Windows VDS
/// (Virtual Disk Service) concept with no direct Linux equivalent.
///
/// On Linux, automount behaviour is controlled by udev rules and /etc/fstab.
/// Use 'automount enable/disable' for the closest equivalent.
pub fn run(args: &[&str], _ctx: &mut Context) {
    if args.is_empty() {
        // Display current "policy" — explain the Linux equivalent
        println!("SAN Policy : (not applicable on Linux)");
        println!();
        println!("The SAN policy is a Windows VDS concept that controls whether new disks");
        println!("are automatically brought online and made writable.");
        println!();
        println!("On Linux, the equivalent is controlled by:");
        println!("  udev rules  : /etc/udev/rules.d/");
        println!("  automount   : 'automount enable' / 'automount disable' in diskparted");
        println!("  fstab       : /etc/fstab (add 'noauto' to prevent auto-mount)");
        return;
    }

    for arg in args {
        if let Some(policy) = arg.strip_prefix("policy=") {
            match policy.to_lowercase().as_str() {
                "onlineall" => {
                    println!("SAN policy 'OnlineAll' is not directly settable on Linux.");
                    println!("Equivalent: ensure no 'noauto' entries in /etc/fstab, and");
                    println!("            udev rules do not block automounting.");
                }
                "offlineall" | "offlineshared" => {
                    println!("SAN policy '{}' is not directly settable on Linux.", policy);
                    println!("Equivalent: add 'noauto' to entries in /etc/fstab, or");
                    println!("            create a udev rule to block automounting.");
                    println!("See: 'automount disable' for the selected partition.");
                }
                _ => println!("Unknown SAN policy: '{}'. Valid values: OnlineAll, OfflineAll, OfflineShared.", policy),
            }
        } else {
            println!("Unknown parameter: '{}'. Usage: san [policy={{onlineall|offlineall|offlineshared}}]", arg);
        }
    }
}
