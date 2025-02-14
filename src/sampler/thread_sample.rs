use super::*;

#[derive(Debug)]
pub struct ThreadSample {
    thread_id: remoteprocess::Tid,
    root_node: SampleNode,
}

impl ThreadSample {
    pub fn new(thread_id: remoteprocess::Tid) -> Self {
        Self {
            thread_id,
            root_node: SampleNode::root_node(),
        }
    }

    /// Generate tree nodes representing the specified backtrace,
    /// and combine with the existing tree by incrementing the counter for common nodes.
    pub fn add_backtrace<'a>(&mut self, backtrace: impl std::iter::Iterator<Item = &'a u64>) {
        self.root_node.increment_count();
        self.root_node.add_backtrace(backtrace);
    }
}

impl std::fmt::Display for ThreadSample {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "Thread {}", self.thread_id)?;
        self.root_node.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_thread_sample() {
        let thread_sample = ThreadSample::new(1);

        assert_eq!(thread_sample.thread_id, 1);
        assert_eq!(thread_sample.root_node.is_root_node(), true);
        assert_eq!(
            format!("{:?}", thread_sample),
            "ThreadSample { thread_id: 1, root_node: \
                SampleNode { level: 0, address: 0, count: 0, children: [] } }"
        );
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
