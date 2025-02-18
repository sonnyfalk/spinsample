#[derive(Debug)]
pub struct SamplePoint {
    level: u32,
    address: u64,
    count: u32,
}

impl SamplePoint {
    pub fn new(level: u32, address: u64) -> Self {
        Self {
            level,
            address,
            count: 1,
        }
    }

    pub fn root_sample() -> Self {
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
    fn test_empty_sample() {
        let sample = SamplePoint::new(1, 2);

        assert_eq!(sample.level, 1);
        assert_eq!(sample.address, 2);
        assert_eq!(sample.count, 1);
    }

    #[test]
    fn test_root_sample() {
        let root_sample = SamplePoint::root_sample();

        assert_eq!(root_sample.count, 0);
        assert_eq!(root_sample.address, 0);
    }
}
