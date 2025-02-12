pub type Pid = remoteprocess::Pid;

pub struct ProcessSample {
    _threads: Vec<ThreadSample>,
}

pub struct ThreadSample {}

impl ProcessSample {
    pub fn profile(pid: Pid) -> Option<ProcessSample> {
        let process = remoteprocess::Process::new(pid).ok()?;
        let _unwinder = process.unwinder().ok()?;

        None
    }
}
