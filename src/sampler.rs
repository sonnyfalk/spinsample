mod error;
mod module_info;
mod process_info;
mod process_sample;
mod raw_sample;
mod remoteprocess_ext;
mod sample_point;
mod symbol_table;
mod thread_sample;

use std::path::PathBuf;

pub use error::Error;
pub use module_info::ModuleInfo;
pub use process_info::ProcessInfo;
pub use process_sample::ProcessSample;
use raw_sample::RawSample;
pub use remoteprocess_ext::*;
pub use sample_point::SamplePoint;
pub use symbol_table::{SymbolInfo, SymbolTable};
pub use thread_sample::ThreadSample;

/// Sample all the threads of the specified process every 10ms.
/// It currently takes up to 500 samples before returning.  
pub fn profile(pid: Pid) -> Result<ProcessSample, Error> {
    // Attach to the specified process and create a callstack unwinder.
    let process = remoteprocess::Process::new(pid).map_err(Error::OpenProcessFailed)?;
    let unwinder = process.unwinder().map_err(Error::OpenProcessFailed)?;
    let symbolicator = process.symbolicator().map_err(Error::OpenProcessFailed)?;
    let process_name = process.exe().ok();

    println!(
        "Sampling process: {} - {}",
        pid,
        process_name.as_ref().unwrap_or(&String::from("{unknown}"))
    );

    let (before_user_time, before_kernel_time) = process.cpu_time();

    // Take backtrace snapshots of all threads in the specified process every 10ms.
    let mut raw_samples = (0..500).fold(Vec::new(), |mut raw_samples, _| {
        if let Some(snapshot) = snapshot(&process, &unwinder).as_mut() {
            raw_samples.append(snapshot);
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
        raw_samples
    });

    let (after_user_time, after_kernel_time) = process.cpu_time();

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

            let mut thread_sample = ThreadSample::new(thread_id);
            for raw_sample in raw_thread_samples {
                thread_sample.add_backtrace(raw_sample.get_backtrace().iter().rev());
            }
            thread_sample
        })
        .collect();

    let modules = process.loaded_modules();

    println!();

    Ok(ProcessSample::new(
        ProcessInfo::new(
            pid,
            PathBuf::from(process_name.unwrap_or_default()),
            modules,
            after_user_time.abs_diff(before_user_time),
            after_kernel_time.abs_diff(before_kernel_time),
        ),
        threads,
        symbol_table,
    ))
}

/// Take a backtrace snapshot of all the threads in the specified process.
fn snapshot(
    process: &remoteprocess::Process,
    unwinder: &remoteprocess::Unwinder,
) -> Option<Vec<RawSample>> {
    let threads = process.threads().ok()?;

    let mut snapshot = Vec::new();
    for thread in threads {
        let Ok(_lock) = thread.lock() else {
            continue;
        };
        let Ok(thread_id) = thread.id() else {
            continue;
        };
        let Ok(cursor) = unwinder.cursor(&thread) else {
            continue;
        };
        let backtrace: Vec<u64> = cursor.flat_map(Result::ok).collect();
        snapshot.push(RawSample::new(thread_id, backtrace));
    }

    Some(snapshot)
}
