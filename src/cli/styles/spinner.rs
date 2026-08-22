use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

/// Static templates only; caller-provided text never enters the template,
/// so braces in prefixes cannot break (or panic) the template parser.
const SPIN_TEMPLATE: &str = "[{spinner}] {msg}";
const FINISH_TEMPLATE: &str = "[{prefix}] {msg}";
const TICK_INTERVAL_MS: u64 = 100;

pub struct Spinner {
    pb: ProgressBar,
}

impl Spinner {
    /// Creates a new spinner and starts the steady tick
    pub fn new(msg: &str) -> Self {
        let pb = ProgressBar::new_spinner();

        let style = ProgressStyle::with_template(SPIN_TEMPLATE)
            .expect("static template must be valid")
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]);

        pb.set_style(style);
        pb.set_message(msg.to_string());
        pb.enable_steady_tick(Duration::from_millis(TICK_INTERVAL_MS));

        Self { pb }
    }

    /// Finishes the spinner with a custom status prefix and message.
    /// Consumes `self` to prevent the spinner from being used after completion.
    pub fn finish(self, prefix: &str, msg: &str) {
        let style =
            ProgressStyle::with_template(FINISH_TEMPLATE).expect("static template must be valid");

        self.pb.set_style(style);
        self.pb.set_prefix(prefix.to_string());
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
