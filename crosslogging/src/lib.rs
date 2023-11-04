use fern::colors::{Color, ColoredLevelConfig};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};

pub fn init_fern_logger() -> anyhow::Result<()> {
    if let Ok(_) = std::env::var("NO_LOG") {
        return Ok(());
    }

    let loglevel = match std::env::var("LOGLEVEL") {
        Ok(v) => match v.as_str() {
            "DEBUG" => log::LevelFilter::Debug,
            "INFO" => log::LevelFilter::Info,
            "OFF" => log::LevelFilter::Off,
            "ERROR" => log::LevelFilter::Error,
            "WARN" => log::LevelFilter::Warn,
            "TRACE" => log::LevelFilter::Trace,
            _ => panic!("Invalid LOGLEVEL input"),
        },
        Err(_) => log::LevelFilter::Info,
    };

    let colors = ColoredLevelConfig::new()
        .info(Color::Green)
        .warn(Color::Yellow)
        .error(Color::Red)
        .trace(Color::White)
        .debug(Color::Blue);

    let mut fern_dis = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "<{}> [{} {}] {}",
                humantime::format_rfc3339(SystemTime::now()),
                colors.color(record.level()),
                record.target(),
                message,
            ))
        })
        .level(loglevel);

    if let Err(_) = std::env::var("NO_STDOUT") {
        fern_dis = fern_dis.chain(std::io::stdout());
    }

    if let Ok(path) = std::env::var("LOGFILE") {
        fern_dis = fern_dis.chain(fern::log_file(path)?);
    }

    if let Ok(path) = std::env::var("LOGDIR") {
        let l = FileDateLogger::new(&path, Duration::from_secs(60))?;
        fern_dis = fern_dis.chain(fern::Output::writer(Box::new(l), "\n"));
    }

    fern_dis.apply()?;
    Ok(())
}

struct FileDateLogger {
    current_file: Arc<Mutex<File>>,
}

impl FileDateLogger {
    fn new(dir_path: &str, dur: Duration) -> std::io::Result<Self> {
        let date = chrono::Local::now().format("%Y-%m-%d");
        let path = format!("{}/{}-crosslive.log", dir_path, date);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .write(true)
            .open(path)?;

        let current_file = Arc::new(Mutex::new(file));

        let og_path = dir_path.to_string();
        let file_mutex = current_file.clone();
        thread::spawn(move || {
            let mut current_date = chrono::Local::now();
            loop {
                thread::sleep(dur);
                let new_date = chrono::Local::now();

                if new_date != current_date {
                    current_date = new_date;
                    let new_file_name = format!(
                        "{}/{}-crosslive.log",
                        og_path,
                        current_date.format("%Y-%m-%d")
                    );

                    let new_file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .write(true)
                        .open(new_file_name)
                        .unwrap();

                    let mut lock = file_mutex.lock().unwrap();
                    *lock = new_file;
                    drop(lock);
                }
            }
        });

        Ok(FileDateLogger { current_file })
    }
}

impl Write for FileDateLogger {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.current_file.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.current_file.lock().unwrap().flush()
    }
}
