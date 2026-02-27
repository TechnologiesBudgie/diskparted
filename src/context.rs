/*
 * DiskParted - A Disk Management Tool
 * Copyright (C) 2026 Raphaël Larocque
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
#[derive(Debug, Clone)]
pub struct Disk {
    pub index: u32,
    pub name: String,      // e.g. "sda"
    pub path: String,      // e.g. "/dev/sda"
    pub size: String,
}

#[derive(Debug, Clone)]
pub struct Partition {
    pub index: u32,
    pub name: String,      // e.g. "sda1"
    pub path: String,      // e.g. "/dev/sda1"
    pub size: String,
}

#[derive(Debug, Default)]
pub struct Context {
    pub selected_disk: Option<Disk>,
    pub selected_partition: Option<Partition>,
}
