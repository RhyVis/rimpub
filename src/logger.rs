use std::io::{self, Write};

use log::{Level, Log, Metadata, Record};

pub struct SimpleColorLogger;

impl Log for SimpleColorLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let level_color = match record.level() {
                Level::Error => "\x1b[31m[ERROR]\x1b[0m", // 红色
                Level::Warn => "\x1b[33m[WARN]\x1b[0m",   // 黄色
                Level::Info => "\x1b[32m[INFO]\x1b[0m",   // 绿色
                Level::Debug => "\x1b[36m[DEBUG]\x1b[0m", // 青色
                Level::Trace => "\x1b[35m[TRACE]\x1b[0m", // 紫色
            };

            println!("{} {}", level_color, record.args());
            io::stdout().flush().unwrap();
        }
    }

    fn flush(&self) {
        io::stdout().flush().unwrap();
    }
}

pub fn init() -> Result<(), log::SetLoggerError> {
    log::set_logger(&SimpleColorLogger)?;
    log::set_max_level(log::LevelFilter::Info);
    Ok(())
}
