#[derive(Debug)]
pub enum Error {
    AttachProcessFailed(windows::core::Error),
    SymInitializeFailed(windows::core::Error),
    BacktraceFailed(windows::core::Error),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::AttachProcessFailed(inner_error) => Some(inner_error),
            Self::SymInitializeFailed(inner_error) => Some(inner_error),
            Self::BacktraceFailed(inner_error) => Some(inner_error),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::AttachProcessFailed(inner_error) => write!(
                f,
                "unable to attach to the specified process for sampling: {}",
                inner_error
            ),
            Self::SymInitializeFailed(inner_error) => {
                write!(f, "unable to initialize symbolication: {}", inner_error)
            }
            Self::BacktraceFailed(inner_error) => {
                write!(f, "unable to capture backtrace: {}", inner_error)
            }
        }
    }
}
