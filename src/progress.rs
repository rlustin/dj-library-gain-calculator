use std::ops::Deref;
use std::sync::Arc;

use crate::logging;

pub use indicatif::{ProgressDrawTarget, ProgressStyle};

pub struct ProgressBar {
    inner: Arc<indicatif::ProgressBar>,
}

impl ProgressBar {
    pub fn new(len: u64) -> Self {
        indicatif::ProgressBar::new(len).into()
    }

    pub fn finish(&self) {
        self.inner.finish();
        logging::set_progress_bar(None);
    }
}

impl From<indicatif::ProgressBar> for ProgressBar {
    fn from(progress_bar: indicatif::ProgressBar) -> Self {
        let inner = Arc::new(progress_bar);
        logging::set_progress_bar(Some(Arc::downgrade(&inner)));
        ProgressBar { inner }
    }
}

impl Deref for ProgressBar {
    type Target = indicatif::ProgressBar;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
