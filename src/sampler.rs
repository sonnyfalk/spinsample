mod error;
mod process_sample;
mod sample_node;
mod thread_sample;

pub use error::Error;
pub use process_sample::ProcessSample;
pub use sample_node::SampleNode;
pub use thread_sample::ThreadSample;

pub type Pid = remoteprocess::Pid;
pub type Tid = remoteprocess::Tid;

/// Sample all the threads of the specified process every 10ms.
/// It currently takes up to 500 samples before returning.  
pub fn profile(pid: Pid) -> Result<ProcessSample, Error> {
    // Attach to the specified process and create a callstack unwinder.
    let process = remoteprocess::Process::new(pid).map_err(Error::OpenProcessFailed)?;
    let unwinder = process.unwinder().map_err(Error::OpenProcessFailed)?;

    println!(
        "Sampling process: {} - {}",
        pid,
        process.exe().unwrap_or("{unknown}".to_string())
    );

    // Take backtrace snapshots of all threads in the specified process every 10ms.
    let mut raw_samples = (0..500).fold(Vec::new(), |mut raw_samples, _| {
        if let Some(snapshot) = snapshot(&process, &unwinder).as_mut() {
            raw_samples.append(snapshot);
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
        raw_samples
    });

    println!("Raw thread samples: {}", raw_samples.len());

    // Sort all raw samples by thread, then iterate through them grouped by thread
    // to produce the final sample tree for each thread.
    raw_samples.sort_by(|a, b| a.thread_id.cmp(&b.thread_id));
    let threads = raw_samples
        .chunk_by(|a, b| a.thread_id == b.thread_id)
        .map(|raw_thread_samples| {
            let thread_id = raw_thread_samples
                .first()
                .map(|raw_sample| raw_sample.thread_id)
                .unwrap();
            let mut thread_sample = ThreadSample::new(thread_id);
            for raw_sample in raw_thread_samples {
                thread_sample.add_backtrace(raw_sample.backtrace.iter().rev());
            }
            thread_sample
        })
        .collect();

    Ok(ProcessSample::new(threads))
}

#[derive(Debug)]
struct RawThreadSample {
    thread_id: Tid,
    backtrace: Vec<u64>,
}

/// Take a backtrace snapshot of all the threads in the specified process.
fn snapshot(
    process: &remoteprocess::Process,
    unwinder: &remoteprocess::Unwinder,
) -> Option<Vec<RawThreadSample>> {
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
        snapshot.push(RawThreadSample {
            thread_id,
            backtrace,
        });
    }

    Some(snapshot)
}
