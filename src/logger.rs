use std::sync::{Mutex, MutexGuard};

use colored::Colorize;

pub struct Logger {
    pub verbose: bool,
}

pub trait Log<'a> {
    fn access(&'a self) -> MutexGuard<'a, Logger>;
    fn log(&'a self, message: &str);
    fn success(&'a self, message: &str);
    fn warning(&'a self, message: &str);
    fn change_verbosity(&'a self, verbose: bool);
}

impl Logger {
    pub const fn new() -> Self {
        Self { verbose: false }
    }
}

impl<'a> Log<'a> for Mutex<Logger> {
    fn access(&'a self) -> MutexGuard<'a, Logger> {
        self.lock().unwrap()
    }

    /// Outputs a message to the terminal. Only shows if the `verbose` flag is set
    fn log(&self, message: &str) {
        let logger = self.access();
        if logger.verbose {
            println!("{}", message);
        }
    }

    /// Outputs a success message
    fn success(&self, message: &str) {
        println!("{} {}", "Success:".cyan().bold(), message);
    }

    /// Outputs a warning message
    fn warning(&self, message: &str) {
        println!("{} {}", "Warning:".yellow().bold(), message);
    }

    fn change_verbosity(&self, verbose: bool) {
        let mut logger = self.access();
        logger.verbose = verbose;
    }
}
