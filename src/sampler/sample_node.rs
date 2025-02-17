#[derive(Debug)]
pub struct SampleNode {
    level: u32,
    address: u64,
    count: u32,
    children: Vec<SampleNode>,
}

impl SampleNode {
    pub fn new(level: u32, address: u64) -> Self {
        Self {
            level,
            address,
            count: 1,
            children: Vec::new(),
        }
    }

    pub fn root_node() -> Self {
        Self {
            level: 0,
            address: 0,
            count: 0,
            children: Vec::new(),
        }
    }

    pub fn increment_count(&mut self) {
        self.count += 1;
    }

    pub fn get_level(&self) -> u32 {
        self.level
    }

    pub fn get_address(&self) -> u64 {
        self.address
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn get_children(&self) -> &Vec<SampleNode> {
        &self.children
    }

    pub fn add_backtrace<'a>(&mut self, mut backtrace: impl std::iter::Iterator<Item = &'a u64>) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_node() {
        let node = SampleNode::new(1, 2);

        assert_eq!(node.level, 1);
        assert_eq!(node.address, 2);
        assert_eq!(node.count, 1);
        assert_eq!(node.children.len(), 0);
    }

    #[test]
    fn test_root_node() {
        let root_node = SampleNode::root_node();

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
}
