use windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_AMD64;

use super::*;

pub struct Backtrace {
    process_handle: HANDLE,
    thread_handle: HANDLE,
    current_frame: STACKFRAME64,
    current_context: CONTEXT,
}

impl Backtrace {
    pub fn backtrace(process_handle: HANDLE, thread_handle: HANDLE) -> Result<Self, Error> {
        let mut current_frame = STACKFRAME64::default();
        let mut current_context = CONTEXT::default();
        current_context.ContextFlags = CONTEXT_FULL_AMD64;

        unsafe {
            SuspendThread(thread_handle);
            GetThreadContext(thread_handle, &mut current_context).map_err(|e| {
                ResumeThread(thread_handle);
                Error::BacktraceFailed(e)
            })?;
        };

        current_frame.AddrStack.Offset = current_context.Rsp;
        current_frame.AddrStack.Mode = AddrModeFlat;
        current_frame.AddrFrame.Offset = current_context.Rbp;
        current_frame.AddrFrame.Mode = AddrModeFlat;
        current_frame.AddrPC.Offset = current_context.Rip;
        current_frame.AddrPC.Mode = AddrModeFlat;

        Ok(Self {
            process_handle,
            thread_handle,
            current_frame,
            current_context,
        })
    }
}

impl Drop for Backtrace {
    fn drop(&mut self) {
        unsafe {
            ResumeThread(self.thread_handle);
        }
    }
}

impl Iterator for Backtrace {
    type Item = STACKFRAME64;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if StackWalk64(
                IMAGE_FILE_MACHINE_AMD64.0 as u32,
                self.process_handle,
                self.thread_handle,
                &mut self.current_frame,
                &raw mut self.current_context as *mut c_void,
                None,
                None,
                None,
                None,
            ) == TRUE
            {
                Some(self.current_frame)
            } else {
                None
            }
        }
    }
}
