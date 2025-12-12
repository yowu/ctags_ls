use std::io::{self, Write};

pub struct Logger;

impl Logger {
    pub fn setup() -> io::Result<()> {
        env_logger::Builder::new()
            .filter(None, log::LevelFilter::Info)
            .format(|buf, record| {
                writeln!(
                    buf,
                    "{} [{}] [PID: {}]- {}",
                    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                    record.level(),
                    std::process::id(),
                    record.args()
                )
            })
            .target(env_logger::Target::Stderr)
            .init();

        Ok(())
    }

    pub fn info(message: &str) {
        log::info!("{}", message);
    }

    pub fn error(message: &str) {
        log::error!("{}", message);
    }
}
