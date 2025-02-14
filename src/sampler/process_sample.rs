use super::*;

#[allow(dead_code)]
#[derive(Debug)]
pub struct ProcessSample {
    threads: Vec<ThreadSample>,
}

impl ProcessSample {
    pub fn new(threads: Vec<ThreadSample>) -> Self {
        Self { threads }
    }
}
