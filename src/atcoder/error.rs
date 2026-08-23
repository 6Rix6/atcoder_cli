use thiserror::Error;

#[derive(Debug, Error)]
pub enum AtcoderError {
    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error("failed to parse problem page ({url})")]
    Parse { url: String },

    #[error("no sample cases found on the problem page ({url}); the page layout may have changed")]
    NoSamples { url: String },

    #[error("no tasks found ({url}); the contest may not publish its task list yet")]
    NoTasksFound { url: String },
}

impl AtcoderError {
    /// Status code of the underlying HTTP error, if any.
    pub fn as_http(&self) -> Option<&reqwest::Error> {
        match self {
            AtcoderError::Http(e) => Some(e),
            _ => None,
        }
    }
}
