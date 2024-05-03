use chrono::Local;
use fern::Dispatch;
use log::LevelFilter;
use std::env;

const DEFAULT_LEVEL: LevelFilter = LevelFilter::Warn;

pub fn setup_logging(username: String) -> Result<(), fern::InitError> {
    let log_file = "logs/app.log";

    let level = match env::var("LOG") {
        Ok(ref s) if s == "error" => LevelFilter::Error,
        Ok(ref s) if s == "warn" => LevelFilter::Warn,
        Ok(ref s) if s == "info" => LevelFilter::Info,
        Ok(ref s) if s == "debug" => LevelFilter::Debug,
        Ok(ref s) if s == "trace" => LevelFilter::Trace,
        Ok(_) => DEFAULT_LEVEL,
        Err(_) => DEFAULT_LEVEL,
    };

    // file based logging
    let file_config = Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{} [{}] [{}] {}",
                Local::now().format("[%Y-%m-%d %H:%M:%S]"),
                username,
                record.level(),
                message
            ))
        })
        .level(level)
        .chain(fern::log_file(log_file)?);

    file_config.apply()?;

    Ok(())
}
