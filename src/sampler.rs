pub struct ProcessSample {
    _threads: Vec<ThreadSample>,
}

pub struct ThreadSample {}

impl ProcessSample {
    pub fn profile(_pid: u64) -> Option<ProcessSample> {
        None
    }
}
