#[derive(PartialEq)]
pub enum AppMode {
    Edit,
    Diff,
}

impl Default for AppMode {
    fn default() -> Self {
        Self::Edit
    }
}