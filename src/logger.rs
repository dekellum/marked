//! A very simple Log output implementation for testing and any CLIs.

use std::error::Error as StdError;
use std::io::Write;

#[cfg(test)]
use std::sync::Once;

/// Conveniently compact type alias for dyn Trait `std::error::Error`.
type Flaw = Box<dyn StdError + Send + Sync + 'static>;

struct Monolog {
    other: log::Level
}

impl log::Log for Monolog {
    fn enabled(&self, meta: &log::Metadata<'_>) -> bool {
        meta.level() <= self.other || meta.target().starts_with("prescan")
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            writeln!(
                std::io::stderr(),
                "{:5} {} {}: {}",
                record.level(), record.target(),
                std::thread::current().name().unwrap_or("-"),
                record.args()
            ).ok();
        }
    }

    fn flush(&self) {
        std::io::stderr().flush().ok();
    }
}

/// Setup logger for a test run, if not already setup, based on TEST_LOG
/// environment variable.
///
/// `TEST_LOG=0` : The default, no logging enabled.
///
/// `TEST_LOG=1` : Info log level.
///
/// `TEST_LOG=2` : Debug log level, Info for deps.
///
/// `TEST_LOG=3` : Debug log level (for all).
///
/// `TEST_LOG=4` : Trace log level, Debug for deps.
///
/// `TEST_LOG=5`+ : Trace log level (for all).
#[cfg(test)]
pub(crate) fn ensure_logger() {
    static TEST_LOG_INIT: Once = Once::new();

    TEST_LOG_INIT.call_once(|| {
        let level = if let Ok(l) = std::env::var("TEST_LOG") {
            l.parse().expect("TEST_LOG parse integer")
        } else {
            0
        };
        if level > 0 {
            setup_logger(level).expect("setup logger");
        }
    });
}

/// Setup logger based on specified level.
///
/// Will fail if already setup.
pub fn setup_logger(level: u32) -> Result<(), Flaw> {
    if level > 0 {
        if level == 1 {
            log::set_max_level(log::LevelFilter::Info)
        } else if level < 4 {
            log::set_max_level(log::LevelFilter::Debug)
        } else {
            log::set_max_level(log::LevelFilter::Trace)
        }

        let other = match level {
            1..=2 => log::Level::Info,
            3..=4 => log::Level::Debug,
            _ => log::Level::Trace, // unfiltered
        };
        log::set_boxed_logger(Box::new(Monolog { other }))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::ensure_logger;
    use log::{debug, trace};

    #[test]
    fn log_setup() {
        ensure_logger();
        debug!("log message");
        trace!("log message 2");
    }
}
