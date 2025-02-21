use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::Diagnostics::ToolHelp::*;

use super::*;

pub struct ProcessIterator {
    snapshot: Owned<HANDLE>,
    current: PROCESSENTRY32W,
}

impl ProcessIterator {
    pub fn snapshot() -> Option<Self> {
        if let Ok(snapshot) = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) } {
            Some(Self {
                snapshot: unsafe { Owned::new(snapshot) },
                current: PROCESSENTRY32W::default(),
            })
        } else {
            None
        }
    }
}

impl Iterator for ProcessIterator {
    type Item = (String, Pid);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.dwSize == 0 {
            self.current.dwSize = size_of::<PROCESSENTRY32W>() as u32;
            unsafe { Process32FirstW(*self.snapshot, &mut self.current).ok()? }
        } else {
            unsafe { Process32NextW(*self.snapshot, &mut self.current).ok()? }
        }

        let process_name = String::from_utf16(&self.current.szExeFile).unwrap_or_default();
        let pid = self.current.th32ProcessID;

        Some((process_name, pid))
    }
}

#[cfg(test)]
mod tests {
    use super::ProcessIterator;

    #[test]
    fn test_process_iterator() {
        let snapshot = ProcessIterator::snapshot().expect("Failed to create process snapshot");
        for (process_name, _pid) in snapshot {
            assert!(!process_name.is_empty());
        }
    }
}
