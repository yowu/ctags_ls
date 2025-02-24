use std::fs::OpenOptions;
use std::io::{self, Write};

pub struct Logger;

impl Logger {
    pub fn setup() -> io::Result<()> {
        let mut log_file_path = std::env::temp_dir();
        log_file_path.push("ctags_ls.log");

        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_path)?;
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
            .target(env_logger::Target::Pipe(Box::new(log_file)))
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
