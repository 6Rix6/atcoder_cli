use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

pub struct Spinner {
    pb: ProgressBar,
}

impl Spinner {
    /// Creates a new spinner and starts the steady tick
    pub fn new(msg: &'static str) -> Self {
        let pb = ProgressBar::new_spinner();
        
        pb.set_style(
            ProgressStyle::with_template(&format_ln!("{spinner:}", "{{msg}}"))
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(msg);
        pb.enable_steady_tick(Duration::from_millis(100));
        
        Self { pb }
    }

    /// Finishes the spinner with a custom status prefix and message.
    /// Consumes `self` to prevent the spinner from being used after completion.
    pub fn finish(self, prefix: &str, msg: &str) {
        self.pb.set_style(ProgressStyle::with_template(&format_ln!(prefix, "{{msg}}")).unwrap());
        self.pb.finish_with_message(msg.to_string());
    }
}

/// Automatically cleans up the spinner if the thread panics or returns early
impl Drop for Spinner {
    fn drop(&mut self) {
        if !self.pb.is_finished() {
            self.pb.finish_and_clear();
        }
    }
}
