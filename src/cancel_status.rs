use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct CancelStatus {
    should_cancel: Arc<AtomicBool>,
}

impl CancelStatus {
    pub fn new() -> Self {
        Self {
            should_cancel: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn activate_ctrl_c_handler(&self) {
        let should_cancel = self.should_cancel.clone();
        _ = ctrlc::set_handler(move || {
            should_cancel.store(true, Ordering::SeqCst);
        });
    }

    pub fn is_canceled(&self) -> bool {
        self.should_cancel.load(Ordering::SeqCst)
    }
}
