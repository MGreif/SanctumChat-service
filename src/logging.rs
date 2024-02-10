use std::{fs::File, path::Path, sync::Arc};

use tracing::Subscriber;
use tracing_subscriber::FmtSubscriber;


pub struct FileLogger {
    path: String,
    pub file: Option<File>
}

impl FileLogger {
    pub fn new(path: String) -> Self {
        return Self { path, file: None }
    }

    pub fn open(&mut self) {
        let exists = Path::new(&self.path).exists();

        if (exists) {
            let access_fd = File::options()
            .append(true)
            .write(true)
            .truncate(false)
            .open(&self.path)
            .expect("Failed to open access log file");
            self.file = Some(access_fd);

        } else {
            let access_fd = File::create(&self.path)
            .expect("Failed to open access log file");
            self.file = Some(access_fd);
        };

    }
}

