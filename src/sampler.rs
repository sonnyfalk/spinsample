use std::ffi::*;
use std::path::PathBuf;
use std::time::Duration;

use windows::core::{Owned, PCWSTR, PWSTR};
use windows::Win32::Foundation::*;
use windows::Win32::System::Diagnostics::Debug::*;
use windows::Win32::System::Threading::*;

use super::cancel_status::*;
use super::*;

mod backtrace;
mod error;
mod module_info;
mod process_info;
mod process_sample;
mod raw_sample;
mod sample_point;
mod symbol_table;
mod symbolicator;
mod thread_sample;

pub use error::Error;
pub use module_info::ModuleInfo;
pub use process_info::ProcessInfo;
pub use process_sample::ProcessSample;
pub use sample_point::SamplePoint;
pub use symbol_table::{SymbolInfo, SymbolTable};
pub use symbolicator::Symbolicator;
pub use thread_sample::ThreadSample;

use backtrace::Backtrace;
use raw_sample::RawSample;

pub type Pid = u32;
pub type Tid = u32;

struct Sampler {
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

        Ok(Sampler { process_handle })
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

    fn thread_cpu_time(&self, thread_handle: HANDLE) -> (std::time::Duration, std::time::Duration) {
        let mut creation_time = FILETIME::default();
        let mut exit_time = FILETIME::default();
        let mut kernel_time = FILETIME::default();
        let mut user_time = FILETIME::default();

        _ = unsafe {
            GetThreadTimes(
                thread_handle,
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
                let (user_cpu_time, kernel_cpu_time) = self.thread_cpu_time(thread_handle);

                snapshot.push(RawSample::new(
                    thread_id,
                    user_cpu_time,
                    kernel_cpu_time,
                    backtrace.map(|frame| frame.AddrPC.Offset).collect(),
                ));
            }
        }

        Ok(snapshot)
    }

    unsafe fn thread_iter(&self) -> impl Iterator<Item = HANDLE> {
        ThreadIterator::new(*self.process_handle)
    }
}

/// Sample all the threads of the specified process at the specified interval.
pub fn profile(pid: Pid, duration: Duration, interval: Duration) -> Result<ProcessSample, Error> {
    let sampler = Sampler::attach(pid)?;

    let symbolicator = Symbolicator::new(*sampler.process_handle)?;

    let exe_file = sampler.exe();

    let cancel_status = CancelStatus::new();
    cancel_status.activate_ctrl_c_handler();

    println!(
        "Sampling process: {} - {} for {} {} with {} millisecond interval",
        pid,
        exe_file
            .as_ref()
            .map(|p| p.to_str())
            .flatten()
            .unwrap_or("{unknown}"),
        duration.as_secs(),
        match duration.as_secs() {
            1 => "second",
            _ => "seconds",
        },
        interval.as_millis()
    );

    let (before_user_time, before_kernel_time) = sampler.process_cpu_time();

    // Take backtrace snapshots of all threads in the specified process.
    let start_time = std::time::Instant::now();
    let mut raw_samples = Vec::new();
    while start_time.elapsed() < duration {
        if cancel_status.is_canceled() {
            println!("^C [interrupted]");
            break;
        }
        run_and_yield_for_duration(interval, || {
            if let Ok(mut snapshot) = unsafe { sampler.snapshot_threads() } {
                raw_samples.append(&mut snapshot);
            }
        });
    }

    let (after_user_time, after_kernel_time) = sampler.process_cpu_time();

    println!("Symbolicating...");
    let symbol_table =
        raw_samples
            .iter()
            .fold(SymbolTable::new(), |mut symbol_table, raw_sample| {
                symbol_table.symbolicate(raw_sample.get_backtrace(), &symbolicator);
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

            let min_user_cpu_time = raw_thread_samples
                .iter()
                .map(RawSample::get_user_cpu_time)
                .min()
                .unwrap_or_default();
            let max_user_cpu_time = raw_thread_samples
                .iter()
                .map(RawSample::get_user_cpu_time)
                .max()
                .unwrap_or_default();

            let min_kernel_cpu_time = raw_thread_samples
                .iter()
                .map(RawSample::get_kernel_cpu_time)
                .min()
                .unwrap_or_default();
            let max_kernel_cpu_time = raw_thread_samples
                .iter()
                .map(RawSample::get_kernel_cpu_time)
                .max()
                .unwrap_or_default();

            let mut thread_sample = ThreadSample::new(
                thread_id,
                min_user_cpu_time.abs_diff(max_user_cpu_time),
                min_kernel_cpu_time.abs_diff(max_kernel_cpu_time),
            );
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

fn run_and_yield_for_duration<F: FnMut()>(duration: Duration, mut f: F) {
    let start = std::time::Instant::now();
    f();
    if duration > Duration::from_millis(20) {
        std::thread::sleep(duration);
    } else {
        while start.elapsed() < duration {
            std::thread::yield_now();
        }
    }
}
