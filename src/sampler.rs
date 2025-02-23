use std::ffi::*;
use std::path::PathBuf;

use windows::core::{Owned, PCWSTR, PWSTR};
use windows::Win32::Foundation::*;
use windows::Win32::System::Diagnostics::Debug::*;
use windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE_AMD64;
use windows::Win32::System::Threading::*;

mod error;
mod module_info;
mod process_info;
mod process_sample;
mod raw_sample;
mod sample_point;
mod symbol_table;
mod thread_sample;

pub use error::Error;
pub use module_info::ModuleInfo;
pub use process_info::ProcessInfo;
pub use process_sample::ProcessSample;
use raw_sample::RawSample;
pub use sample_point::SamplePoint;
pub use symbol_table::{SymbolInfo, SymbolTable};
pub use thread_sample::ThreadSample;

pub type Pid = u32;
pub type Tid = u32;

struct Sampler {
    symbolicator: Symbolicator,
    process_handle: Owned<HANDLE>,
}

impl Sampler {
    fn attach(pid: Pid) -> Result<Sampler, Error> {
        let process_handle = unsafe {
            Owned::new(
                OpenProcess(
                    PROCESS_VM_READ | PROCESS_SUSPEND_RESUME | PROCESS_QUERY_INFORMATION,
                    false,
                    pid,
                )
                .map_err(|e| Error::AttachProcessFailed(e))?,
            )
        };
        let symbolicator = Symbolicator::new(*process_handle)?;

        Ok(Sampler {
            symbolicator,
            process_handle,
        })
    }

    fn exe(&self) -> Option<PathBuf> {
        unsafe {
            let mut exe_file: [u16; MAX_PATH as usize] = std::mem::zeroed();
            let mut size = MAX_PATH;
            if QueryFullProcessImageNameW(
                *self.process_handle,
                PROCESS_NAME_WIN32,
                PWSTR::from_raw(exe_file.as_mut_ptr()),
                &mut size,
            )
            .is_ok()
            {
                let path_string = PCWSTR::from_raw(exe_file.as_ptr()).to_string().ok()?;
                Some(PathBuf::from(path_string))
            } else {
                None
            }
        }
    }

    fn process_cpu_time(&self) -> (std::time::Duration, std::time::Duration) {
        let mut creation_time = FILETIME::default();
        let mut exit_time = FILETIME::default();
        let mut kernel_time = FILETIME::default();
        let mut user_time = FILETIME::default();

        _ = unsafe {
            GetProcessTimes(
                *self.process_handle,
                &mut creation_time,
                &mut exit_time,
                &mut kernel_time,
                &mut user_time,
            )
        };

        let user_time = std::time::Duration::from_nanos(
            ((user_time.dwHighDateTime as u64) << 32) | (user_time.dwLowDateTime as u64),
        );
        let kernel_time = std::time::Duration::from_nanos(
            ((kernel_time.dwHighDateTime as u64) << 32) | (kernel_time.dwLowDateTime as u64),
        );

        (user_time, kernel_time)
    }

    fn loaded_modules(&self) -> Vec<ModuleInfo> {
        extern "system" fn callback(
            module_name: PCWSTR,
            base_address: u64,
            size: u32,
            modules_ptr: *const c_void,
        ) -> BOOL {
            if let Ok(module_name) = unsafe { module_name.to_string() } {
                let mut modules = unsafe { Box::from_raw(modules_ptr as *mut Vec<ModuleInfo>) };
                modules.push(ModuleInfo::new(
                    PathBuf::from(module_name),
                    base_address,
                    size,
                ));
                Box::leak(modules);
            }
            BOOL::from(true)
        }

        unsafe {
            let modules = Box::new(Vec::<ModuleInfo>::new());
            let modules_ptr = Box::into_raw(modules);
            _ = EnumerateLoadedModulesW64(
                *self.process_handle,
                Some(callback),
                Some(modules_ptr as *mut c_void),
            );
            *Box::from_raw(modules_ptr)
        }
    }

    unsafe fn snapshot_threads(&self) -> Result<Vec<RawSample>, Error> {
        let mut snapshot = Vec::new();

        for thread_handle in self.thread_iter() {
            let thread_id = GetThreadId(thread_handle);
            if let Ok(backtrace) = Backtrace::backtrace(*self.process_handle, thread_handle) {
                snapshot.push(RawSample::new(
                    thread_id,
                    backtrace.map(|frame| frame.AddrPC.Offset).collect(),
                ));
            } else {
                println!("Error capturing backtrace for {}", thread_id);
            }
        }

        Ok(snapshot)
    }

    unsafe fn thread_iter(&self) -> impl Iterator<Item = HANDLE> {
        ThreadIterator::new(*self.process_handle)
    }
}

struct ThreadIterator {
    process_handle: HANDLE,
    current: Owned<HANDLE>,
}

impl ThreadIterator {
    fn new(process_handle: HANDLE) -> Self {
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

struct Backtrace {
    process_handle: HANDLE,
    thread_handle: HANDLE,
    current_frame: STACKFRAME64,
    current_context: CONTEXT,
}

impl Backtrace {
    fn backtrace(process_handle: HANDLE, thread_handle: HANDLE) -> Result<Self, Error> {
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

pub struct Symbolicator {
    process_handle: HANDLE,
}

pub struct SymbolicatedFrame {
    pub function: Option<String>,
    pub module: Option<String>,
}

impl Symbolicator {
    fn new(process_handle: HANDLE) -> Result<Self, Error> {
        unsafe {
            SymInitializeW(process_handle, PCWSTR::null(), true)
                .map_err(|e| Error::SymInitializeFailed(e))?
        };
        Ok(Self { process_handle })
    }

    pub fn symbolicate(&self, address: u64) -> SymbolicatedFrame {
        let function = unsafe {
            let mut displacement: u64 = 0;
            let mut symbol_info = SYMBOL_INFO_PACKAGEW::default();
            symbol_info.si.SizeOfStruct = size_of::<SYMBOL_INFOW>() as u32;
            symbol_info.si.MaxNameLen = MAX_SYM_NAME;

            if SymFromAddrW(
                self.process_handle,
                address,
                Some(&mut displacement),
                &mut symbol_info.si,
            )
            .is_ok()
            {
                PCWSTR::from_raw(symbol_info.si.Name.as_ptr())
                    .to_string()
                    .ok()
            } else {
                None
            }
        };

        let module = unsafe {
            let mut module_info = IMAGEHLP_MODULEW64::default();
            module_info.SizeOfStruct = size_of::<IMAGEHLP_MODULEW64>() as u32;

            if SymGetModuleInfoW64(self.process_handle, address, &mut module_info).is_ok() {
                PCWSTR::from_raw(module_info.ModuleName.as_ptr())
                    .to_string()
                    .ok()
            } else {
                None
            }
        };

        SymbolicatedFrame { function, module }
    }
}

impl Drop for Symbolicator {
    fn drop(&mut self) {
        unsafe {
            _ = SymCleanup(self.process_handle);
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

/// Sample all the threads of the specified process every 10ms.
/// It currently takes up to 500 samples before returning.  
pub fn profile(pid: Pid) -> Result<ProcessSample, Error> {
    let sampler = Sampler::attach(pid)?;
    let exe_file = sampler.exe();

    println!(
        "Sampling process: {} - {}",
        pid,
        exe_file
            .as_ref()
            .map(|p| p.to_str())
            .flatten()
            .unwrap_or("{unknown}")
    );

    let (before_user_time, before_kernel_time) = sampler.process_cpu_time();

    // Take backtrace snapshots of all threads in the specified process every 10ms.
    let mut raw_samples = (0..500).fold(Vec::new(), |mut raw_samples, _| {
        if let Ok(mut snapshot) = unsafe { sampler.snapshot_threads() } {
            raw_samples.append(&mut snapshot);
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
        raw_samples
    });

    let (after_user_time, after_kernel_time) = sampler.process_cpu_time();

    println!("Symbolicating...");
    let symbol_table =
        raw_samples
            .iter()
            .fold(SymbolTable::new(), |mut symbol_table, raw_sample| {
                symbol_table.symbolicate(raw_sample.get_backtrace(), &sampler.symbolicator);
                symbol_table
            });

    println!("Building sample tree...");

    // Sort all raw samples by thread, then iterate through them grouped by thread
    // to produce the final sample tree for each thread.
    raw_samples.sort_by(|a, b| a.get_thread_id().cmp(&b.get_thread_id()));
    let threads = raw_samples
        .chunk_by(|a, b| a.get_thread_id() == b.get_thread_id())
        .map(|raw_thread_samples| {
            let thread_id = raw_thread_samples
                .first()
                .map(|raw_sample| raw_sample.get_thread_id())
                .unwrap();

            let mut thread_sample = ThreadSample::new(thread_id);
            for raw_sample in raw_thread_samples {
                thread_sample.add_backtrace(raw_sample.get_backtrace().iter().rev());
            }
            thread_sample
        })
        .collect();

    let modules = sampler.loaded_modules();

    println!();

    Ok(ProcessSample::new(
        ProcessInfo::new(
            pid,
            exe_file.unwrap_or_default(),
            modules,
            after_user_time.abs_diff(before_user_time),
            after_kernel_time.abs_diff(before_kernel_time),
        ),
        threads,
        symbol_table,
    ))
}
