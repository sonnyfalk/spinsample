use std::time::Duration;

use super::*;

#[derive(Debug)]
pub struct RawSample {
    thread_id: Tid,
    user_cpu_time: Duration,
    kernel_cpu_time: Duration,
    backtrace: Vec<u64>,
}

impl RawSample {
    pub fn new(
        thread_id: Tid,
        user_cpu_time: Duration,
        kernel_cpu_time: Duration,
        backtrace: Vec<u64>,
    ) -> Self {
        Self {
            thread_id,
            user_cpu_time,
            kernel_cpu_time,
            backtrace,
        }
    }

    pub fn get_thread_id(&self) -> Tid {
        self.thread_id
    }

    pub fn get_user_cpu_time(&self) -> Duration {
        self.user_cpu_time
    }

    pub fn get_kernel_cpu_time(&self) -> Duration {
        self.kernel_cpu_time
    }

    pub fn get_backtrace(&self) -> &Vec<u64> {
        &self.backtrace
    }
}
