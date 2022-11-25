use std::io;

mod config;
pub use config::*;
use tracing_subscriber::{fmt, prelude::__tracing_subscriber_SubscriberExt, EnvFilter};

pub fn init<T, E>(config: LoggerConfig, fun: T) -> Result<(), E>
where
    T: FnOnce() -> Result<(), E>,
{
    // Grabs from RUST_LOG env var and if not defaults to
    // TRACE for debug, and info for non debug.
    let level_filter = EnvFilter::from_default_env();
    #[cfg(debug_assertions)]
    let level_filter = level_filter.add_directive(tracing::Level::TRACE.into());
    #[cfg(not(debug_assertions))]
    let level_filter = level_filter.add_directive(tracing::Level::INFO.into());

    let subscriber = tracing_subscriber::registry().with(level_filter);

    let stdout = fmt::Layer::new().with_writer(io::stdout).pretty();
    // Logging shows files and line number but only for debug builds.
    #[cfg(not(debug_assertions))]
    let stdout = stdout.with_line_number(false).with_file(false);
    let subscriber = subscriber.with(stdout);

    if let Some(log_file_path) = config.log_file_path {
        // Log Rotation set to daily.
        let file_appender = tracing_appender::rolling::daily(log_file_path, "example.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        let mut file_logger = fmt::Layer::new().with_writer(non_blocking);
        file_logger.set_ansi(false);
        let subscriber = subscriber.with(file_logger);
        tracing::subscriber::with_default(subscriber, fun)
    } else {
        tracing::subscriber::with_default(subscriber, fun)
    }
}