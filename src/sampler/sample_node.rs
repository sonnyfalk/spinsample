#[derive(Debug)]
pub struct SampleNode {
    level: u32,
    address: u64,
    count: u32,
}

impl SampleNode {
    pub fn new(level: u32, address: u64) -> Self {
        Self {
            level,
            address,
            count: 1,
        }
    }

    pub fn root_node() -> Self {
        Self {
            level: 0,
            address: 0,
            count: 0,
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
    }

    #[test]
    fn test_root_node() {
        let root_node = SampleNode::root_node();

        assert_eq!(root_node.count, 0);
        assert_eq!(root_node.address, 0);
    }
}
