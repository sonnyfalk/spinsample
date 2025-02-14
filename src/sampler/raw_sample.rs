use super::*;

#[derive(Debug)]
pub struct RawSample {
    thread_id: Tid,
    backtrace: Vec<u64>,
}

impl RawSample {
    pub fn new(thread_id: Tid, backtrace: Vec<u64>) -> Self {
        Self {
            thread_id,
            backtrace,
        }
    }

    pub fn get_thread_id(&self) -> Tid {
        self.thread_id
    }

    pub fn get_backtrace(&self) -> &Vec<u64> {
        &self.backtrace
    }
}
