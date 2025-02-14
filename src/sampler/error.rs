#[derive(Debug)]
pub enum Error {
    OpenProcessFailed(remoteprocess::Error),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::OpenProcessFailed(inner_error) => Some(inner_error),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::OpenProcessFailed(inner_error) => write!(
                f,
                "unable to open the specified process for sampling: {}",
                inner_error
            ),
        }
    }
}
