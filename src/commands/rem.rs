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

/// REM — comment line. Silently ignores the rest of the line.
///
/// DiskPart syntax:  rem <any text>
///
/// Used in DiskPart scripts to add comments. This is a pure no-op.
pub fn run(_args: &[&str], _ctx: &mut Context) {
    // Comments produce no output and take no action — this is correct behaviour.
}
