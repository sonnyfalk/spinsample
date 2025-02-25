use std::path::PathBuf;

#[derive(Debug)]
pub struct ModuleInfo {
    path: PathBuf,
    base_address: u64,
    size: u32,
}

impl ModuleInfo {
    pub fn new(path: PathBuf, base_address: u64, size: u32) -> Self {
        Self {
            path,
            base_address,
            size,
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.path.file_name().map(std::ffi::OsStr::to_str).flatten()
    }

    pub fn file_path(&self) -> Option<&str> {
        self.path.to_str()
    }

    pub fn module_dir(&self) -> Option<&str> {
        self.path.parent()?.to_str()
    }

    pub fn address_range(&self) -> std::ops::Range<u64> {
        self.base_address..(self.base_address + self.size as u64)
    }
}
