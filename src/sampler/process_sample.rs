use super::*;

#[derive(Debug)]
pub struct ProcessSample {
    threads: Vec<ThreadSample>,
}

impl ProcessSample {
    pub fn new(threads: Vec<ThreadSample>) -> Self {
        Self { threads }
    }
}

impl std::fmt::Display for ProcessSample {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "Process")?;
        for thread in &self.threads {
            thread.fmt(f)?;
        }
        Ok(())
    }
}
