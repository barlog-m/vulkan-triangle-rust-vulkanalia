pub fn logger_init() {
    std::panic::set_hook(Box::new(|panic_info| {
        log::error!("{}", panic_info);
    }));

    use env_logger::fmt::style::AnsiColor;
    use std::io::Write;

    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            let level = record.level();

            let style = match level {
                log::Level::Error => AnsiColor::Red.on_default().bold(),
                log::Level::Warn => AnsiColor::Yellow.on_default(),
                log::Level::Info => AnsiColor::White.on_default(),
                log::Level::Debug => AnsiColor::BrightBlack.on_default(),
                log::Level::Trace => AnsiColor::BrightBlack.on_default(),
            };

            if level == log::Level::Trace || level == log::Level::Debug {
                writeln!(buf, "{style}{level:<5} {}{style:#}", record.args())
            } else {
                writeln!(buf, "{style}{level:<5}{style:#} {}", record.args())
            }
        })
        .filter_level(log::LevelFilter::Debug)
        .init();
}