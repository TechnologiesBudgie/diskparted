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
