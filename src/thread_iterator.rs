use windows::core::Owned;
use windows::Win32::Foundation::*;

pub struct ThreadIterator {
    process_handle: HANDLE,
    current: Owned<HANDLE>,
}

impl ThreadIterator {
    pub fn new(process_handle: HANDLE) -> Self {
        Self {
            process_handle,
            current: Owned::default(),
        }
    }
}

impl Iterator for ThreadIterator {
    type Item = HANDLE;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let mut new_thread = HANDLE::default();
            if NtGetNextThread(
                self.process_handle,
                *self.current,
                0x2000000,
                0,
                0,
                &mut new_thread,
            ) == NTSTATUS(0)
            {
                self.current = Owned::new(new_thread);
                Some(*self.current)
            } else {
                None
            }
        }
    }
}

#[link(name = "ntdll")]
extern "system" {
    fn NtGetNextThread(
        process: HANDLE,
        thread: HANDLE,
        access: u32,
        attributes: u32,
        flags: u32,
        new_thread: *mut HANDLE,
    ) -> NTSTATUS;
}
