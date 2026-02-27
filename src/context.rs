#[derive(Debug, Clone)]
pub struct Disk {
    pub index: u32,
    pub name: String,      // e.g. "sda"
    pub path: String,      // e.g. "/dev/sda"
    pub size: String,
}

#[derive(Debug, Default)]
pub struct Context {
    pub selected_disk: Option<Disk>,
}