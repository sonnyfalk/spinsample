pub type Pid = remoteprocess::Pid;

pub struct ProcessSample {
    _threads: Vec<ThreadSample>,
}

pub struct ThreadSample {}

struct RawThreadSample {
    _thread_id: remoteprocess::Tid,
    _backtrace: Vec<u64>,
}

#[derive(Debug)]
pub enum Error {
    OpenProcessFailed(remoteprocess::Error),
}

impl ProcessSample {
    /// Sample all the threads of the specified process every 10ms.
    /// It currently takes up to 500 samples before returning.  
    pub fn profile(pid: Pid) -> Result<ProcessSample, Error> {
        let process = remoteprocess::Process::new(pid).map_err(Error::OpenProcessFailed)?;
        let unwinder = process.unwinder().map_err(Error::OpenProcessFailed)?;

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

        Ok(ProcessSample {
            _threads: Vec::new(),
        })
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

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::OpenProcessFailed(inner_error) => Some(inner_error),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::OpenProcessFailed(inner_error) => write!(
                f,
                "unable to open the specified process for sampling: {}",
                inner_error
            ),
        }
    }
}
