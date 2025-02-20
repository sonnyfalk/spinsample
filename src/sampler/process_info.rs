use std::{path::PathBuf, time::Duration};

use super::*;
#[derive(Debug)]
pub struct ProcessInfo {
    pub pid: Pid,
    pub path: PathBuf,
    pub modules: Vec<ModuleInfo>,
    pub user_cpu_time: Duration,
    pub kernel_cpu_time: Duration,
}

impl ProcessInfo {
    pub fn new(
        pid: Pid,
        path: PathBuf,
        modules: Vec<ModuleInfo>,
        user_cpu_time: Duration,
        kernel_cpu_time: Duration,
    ) -> Self {
        Self {
            pid,
            path,
            modules,
            user_cpu_time,
            kernel_cpu_time,
        }
    }
}
