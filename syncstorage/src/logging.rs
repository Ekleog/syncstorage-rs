use std::io;

use crate::error::ApiResult;

use slog::{self, slog_o, Drain};
use slog_mozlog_json::MozLogJson;

pub fn init_logging(json: bool) -> ApiResult<()> {
    let logger = if json {
        let hostname = hostname::get()
            .expect("Couldn't get hostname")
            .into_string()
            .expect("Couldn't get hostname");

        let drain = MozLogJson::new(io::stdout())
            .logger_name(format!(
                "{}-{}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            ))
            .msg_type(format!("{}:log", env!("CARGO_PKG_NAME")))
            .hostname(hostname)
            .build()
            .fuse();
        let drain = slog_envlogger::new(drain);
        let drain = slog_async::Async::new(drain).build().fuse();
        slog::Logger::root(drain, slog_o!())
    } else {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_envlogger::new(drain);
        let drain = slog_async::Async::new(drain).build().fuse();
        slog::Logger::root(drain, slog_o!())
    };
    // With the fix landed by https://github.com/slog-rs/slog/issues/169
    // we normally need to pass the guard back up to keep it in scope for the app.
    // Unfortunately, testing breaks due to guards not being set, or conflicting
    // guards being created, so we still need to initialize and cancel the reset here.
    let guard = slog_scope::set_global_logger(logger);
    slog_stdlog::init().ok();
    guard.cancel_reset();
    Ok(())
}

pub fn reset_logging() {
    let logger = slog::Logger::root(slog::Discard, slog_o!());
    slog_scope::set_global_logger(logger).cancel_reset();
}
