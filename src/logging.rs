use chrono::Local;
use fern::Dispatch;

pub fn setup_logging(username: String) -> Result<(), fern::InitError> {
    let log_file = "logs/app.log";

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
        .level(log::LevelFilter::Info)
        .chain(fern::log_file(log_file)?);

    file_config.apply()?;

    Ok(())
}
