pub type Pid = remoteprocess::Pid;

pub struct ProcessSample {
    _threads: Vec<ThreadSample>,
}

pub struct ThreadSample {}

struct RawThreadSample {
    _thread_id: remoteprocess::Tid,
    _backtrace: Vec<u64>,
}

impl ProcessSample {
    pub fn profile(pid: Pid) -> Option<ProcessSample> {
        let process = remoteprocess::Process::new(pid).ok()?;
        let unwinder = process.unwinder().ok()?;

        println!(
            "Sampling process: {} - {}",
            pid,
            process.exe().unwrap_or("{unknown}".to_string())
        );

        let samples: Vec<RawThreadSample> = (0..500).fold(Vec::new(), |mut samples, _| {
            if let Some(snapshot) = Self::snapshot(&process, &unwinder).as_mut() {
                samples.append(snapshot);
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
            samples
        });

        println!("Raw thread samples: {}", samples.len());

        None
    }

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
            let Ok(_thread_id) = thread.id() else {
                continue;
            };
            let Ok(cursor) = unwinder.cursor(&thread) else {
                continue;
            };
            let _backtrace: Vec<u64> = cursor.flat_map(Result::ok).collect();
            snapshot.push(RawThreadSample {
                _thread_id,
                _backtrace,
            });
        }

        Some(snapshot)
    }
}
