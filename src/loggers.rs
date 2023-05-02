use std::{
    fs::File,
    io::{BufWriter, Write},
    sync::Mutex,
};

pub struct SimpleLogger;
impl log::Log for SimpleLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        println!("{}", record.args());
    }

    fn flush(&self) {}
}

pub struct FileLogger {
    writer: Mutex<BufWriter<File>>,
}

impl FileLogger {
    pub fn new(filename: &str) -> Self {
        let file = File::create(filename).unwrap();
        let writer = Mutex::new(BufWriter::new(file));
        Self { writer }
    }
}

impl log::Log for FileLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }
    fn log(&self, record: &log::Record) {
        let mut writer = self.writer.lock().unwrap();
        writeln!(writer, "{}", record.args()).unwrap();
    }
    fn flush(&self) {
        self.writer.lock().unwrap().flush().unwrap();
    }
}

#[derive(Default)]
pub struct DeferredLogger {
    buf: Mutex<Vec<u8>>,
}

impl DeferredLogger {
    pub const fn new() -> Self {
        Self {
            buf: Mutex::new(Vec::new()),
        }
    }
}

impl log::Log for DeferredLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let mut buf = self.buf.lock().unwrap();
        writeln!(buf, "{}", record.args()).unwrap();
    }

    fn flush(&self) {
        let out = self.buf.lock().map(|buf| String::from_utf8(buf.clone()));
        match out {
            Ok(Ok(logs)) => print!("{logs}"),
            _ => println!("Log file corrupted :("),
        };
    }
}
