use tree_iterators_rs::prelude::*;

use super::*;

#[derive(Debug)]
pub struct ThreadSample {
    thread_id: Tid,
    sample_tree: TreeNode<SamplePoint>,
}

impl ThreadSample {
    pub fn new(thread_id: Tid) -> Self {
        Self {
            thread_id,
            sample_tree: TreeNode {
                value: SamplePoint::root_sample(),
                children: Vec::new(),
            },
        }
    }

    pub fn get_thread_id(&self) -> Tid {
        self.thread_id
    }

    /// Generate sample tree representing the specified backtrace,
    /// and combine with the existing tree by incrementing the counter for common nodes.
    pub fn add_backtrace<'a>(&mut self, backtrace: impl Iterator<Item = &'a u64>) {
        self.sample_tree.value.increment_count();
        add_backtrace(&mut self.sample_tree, backtrace);
    }

    pub fn sample_tree_dfs_iter(&self) -> impl Iterator<Item = &SamplePoint> {
        self.sample_tree.dfs_preorder_iter()
    }
}

fn add_backtrace<'a>(
    node: &mut TreeNode<SamplePoint>,
    mut backtrace: impl Iterator<Item = &'a u64>,
) {
    let Some(&address) = backtrace.next() else {
        return;
    };

    if let Some(node) = node
        .children
        .iter_mut()
        .find(|n| n.value.get_address() == address)
    {
        node.value.increment_count();
        add_backtrace(node, backtrace);
    } else {
        let mut child_node = TreeNode {
            value: SamplePoint::new(node.value.get_level() + 1, address),
            children: Vec::new(),
        };
        add_backtrace(&mut child_node, backtrace);
        node.children.push(child_node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_thread_sample() {
        let thread_sample = ThreadSample::new(1);

        assert_eq!(thread_sample.thread_id, 1);
        assert_eq!(
            format!("{:?}", thread_sample),
            "ThreadSample { thread_id: 1, sample_tree: \
                TreeNode { value: SamplePoint { level: 0, address: 0, count: 0 }, children: [] } }"
        );
    }

    #[test]
    fn test_thread_sample_with_backtrace() {
        let mut thread_sample = ThreadSample::new(1);

        thread_sample.add_backtrace([1, 2, 3].iter());

        assert_eq!(
            format!("{:?}", thread_sample),
            "ThreadSample { thread_id: 1, sample_tree: \
                TreeNode { value: SamplePoint { level: 0, address: 0, count: 1 }, children: [\
                    TreeNode { value: SamplePoint { level: 1, address: 1, count: 1 }, children: [\
                        TreeNode { value: SamplePoint { level: 2, address: 2, count: 1 }, children: [\
                            TreeNode { value: SamplePoint { level: 3, address: 3, count: 1 }, children: [] }] }] }] } }"
        );
    }

    #[test]
    fn test_thread_sample_backtrace_double() {
        let mut thread_sample = ThreadSample::new(1);

        thread_sample.add_backtrace([1, 2].iter());
        thread_sample.add_backtrace([1, 2].iter());

        assert_eq!(format!("{:?}", thread_sample), "ThreadSample { thread_id: 1, sample_tree: \
            TreeNode { value: SamplePoint { level: 0, address: 0, count: 2 }, children: [\
                TreeNode { value: SamplePoint { level: 1, address: 1, count: 2 }, children: [\
                    TreeNode { value: SamplePoint { level: 2, address: 2, count: 2 }, children: [] }] }] } }");
    }

    #[test]
    fn test_thread_sample_backtrace_fork() {
        let mut thread_sample = ThreadSample::new(1);

        thread_sample.add_backtrace([1, 2].iter());
        thread_sample.add_backtrace([1, 3].iter());

        assert_eq!(format!("{:?}", thread_sample), "ThreadSample { thread_id: 1, sample_tree: \
            TreeNode { value: SamplePoint { level: 0, address: 0, count: 2 }, children: [\
                TreeNode { value: SamplePoint { level: 1, address: 1, count: 2 }, children: [\
                    TreeNode { value: SamplePoint { level: 2, address: 2, count: 1 }, children: [] }, \
                    TreeNode { value: SamplePoint { level: 2, address: 3, count: 1 }, children: [] }] }] } }");
    }
}
