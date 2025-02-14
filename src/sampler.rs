pub type Pid = remoteprocess::Pid;

#[allow(dead_code)]
#[derive(Debug)]
pub struct ProcessSample {
    threads: Vec<ThreadSample>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct ThreadSample {
    thread_id: remoteprocess::Tid,
    root_node: SampleNode,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct SampleNode {
    level: u32,
    address: u64,
    count: u32,
    children: Vec<SampleNode>,
}

#[allow(dead_code)]
#[derive(Debug)]
struct RawThreadSample {
    thread_id: remoteprocess::Tid,
    backtrace: Vec<u64>,
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

        let mut raw_samples = (0..500).fold(Vec::new(), |mut raw_samples, _| {
            if let Some(snapshot) = Self::snapshot(&process, &unwinder).as_mut() {
                raw_samples.append(snapshot);
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
            raw_samples
        });

        println!("Raw thread samples: {}", raw_samples.len());

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

        Ok(ProcessSample { threads })
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
}

#[allow(dead_code)]
impl ThreadSample {
    fn new(thread_id: remoteprocess::Tid) -> Self {
        Self {
            thread_id,
            root_node: SampleNode::root_node(),
        }
    }

    fn add_backtrace<'a>(&mut self, backtrace: impl std::iter::Iterator<Item = &'a u64>) {
        self.root_node.increment_count();
        self.root_node.add_backtrace(backtrace);
    }
}

#[allow(dead_code)]
impl SampleNode {
    fn new(level: u32, address: u64) -> Self {
        Self {
            level,
            address,
            count: 1,
            children: Vec::new(),
        }
    }

    fn root_node() -> Self {
        Self {
            level: 0,
            address: 0,
            count: 0,
            children: Vec::new(),
        }
    }

    fn is_root_node(&self) -> bool {
        self.level == 0 && self.address == 0
    }

    fn increment_count(&mut self) {
        self.count += 1;
    }

    fn add_backtrace<'a>(&mut self, mut backtrace: impl std::iter::Iterator<Item = &'a u64>) {
        let Some(&address) = backtrace.next() else {
            return;
        };

        if let Some(node) = self.children.iter_mut().find(|n| n.address == address) {
            node.increment_count();
            node.add_backtrace(backtrace);
        } else {
            let mut node = SampleNode::new(self.level + 1, address);
            node.add_backtrace(backtrace);
            self.children.push(node);
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_node() {
        let node = SampleNode::new(1, 2);

        assert_eq!(node.is_root_node(), false);
        assert_eq!(node.level, 1);
        assert_eq!(node.address, 2);
        assert_eq!(node.count, 1);
        assert_eq!(node.children.len(), 0);
    }

    #[test]
    fn test_root_node() {
        let root_node = SampleNode::root_node();

        assert_eq!(root_node.is_root_node(), true);
        assert_eq!(root_node.count, 0);
        assert_eq!(root_node.address, 0);
        assert_eq!(root_node.children.len(), 0);
    }

    #[test]
    fn test_node_backtrace() {
        let mut root_node = SampleNode::root_node();
        root_node.increment_count();
        root_node.add_backtrace([1, 2].iter());

        assert_eq!(root_node.count, 1);
        assert_eq!(root_node.children.len(), 1);

        let node = &root_node.children[0];
        assert_eq!(node.address, 1);
        assert_eq!(node.level, 1);
        assert_eq!(node.count, 1);
        assert_eq!(node.children.len(), 1);

        let node = &node.children[0];
        assert_eq!(node.address, 2);
        assert_eq!(node.level, 2);
        assert_eq!(node.count, 1);
        assert_eq!(node.children.len(), 0);
    }

    #[test]
    fn test_node_backtrace_double() {
        let mut root_node = SampleNode::root_node();
        root_node.increment_count();
        root_node.add_backtrace([1, 2].iter());
        root_node.increment_count();
        root_node.add_backtrace([1, 2].iter());

        assert_eq!(root_node.count, 2);
        assert_eq!(root_node.children.len(), 1);

        let node = &root_node.children[0];
        assert_eq!(node.address, 1);
        assert_eq!(node.level, 1);
        assert_eq!(node.count, 2);
        assert_eq!(node.children.len(), 1);

        let node = &node.children[0];
        assert_eq!(node.address, 2);
        assert_eq!(node.level, 2);
        assert_eq!(node.count, 2);
        assert_eq!(node.children.len(), 0);
    }
    #[test]
    fn test_node_backtrace_fork() {
        let mut root_node = SampleNode::root_node();
        root_node.increment_count();
        root_node.add_backtrace([1, 2].iter());
        root_node.increment_count();
        root_node.add_backtrace([1, 3].iter());

        assert_eq!(root_node.count, 2);
        assert_eq!(root_node.children.len(), 1);

        let node = &root_node.children[0];
        assert_eq!(node.address, 1);
        assert_eq!(node.level, 1);
        assert_eq!(node.count, 2);
        assert_eq!(node.children.len(), 2);

        let node1 = &node.children[0];
        assert_eq!(node1.address, 2);
        assert_eq!(node1.level, 2);
        assert_eq!(node1.count, 1);
        assert_eq!(node1.children.len(), 0);

        let node2 = &node.children[1];
        assert_eq!(node2.address, 3);
        assert_eq!(node2.level, 2);
        assert_eq!(node2.count, 1);
        assert_eq!(node2.children.len(), 0);
    }

    #[test]
    fn test_empty_thread_sample() {
        let thread_sample = ThreadSample::new(1);

        assert_eq!(thread_sample.thread_id, 1);
        assert_eq!(thread_sample.root_node.is_root_node(), true);
        assert_eq!(thread_sample.root_node.address, 0);
        assert_eq!(thread_sample.root_node.count, 0);
        assert_eq!(thread_sample.root_node.level, 0);
        assert_eq!(thread_sample.root_node.children.len(), 0);
    }

    #[test]
    fn test_thread_sample_with_backtrace() {
        let mut thread_sample = ThreadSample::new(1);

        thread_sample.add_backtrace([1, 2, 3].iter());

        assert_eq!(
            format!("{:?}", thread_sample),
            "ThreadSample { thread_id: 1, root_node: \
                SampleNode { level: 0, address: 0, count: 1, children: [\
                    SampleNode { level: 1, address: 1, count: 1, children: [\
                        SampleNode { level: 2, address: 2, count: 1, children: [\
                            SampleNode { level: 3, address: 3, count: 1, children: [] }] }] }] } }"
        );
    }
}
