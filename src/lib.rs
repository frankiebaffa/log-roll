use {
    chrono::Datelike,
    std::io::Write,
};
enum LogType {
    Info,
    Warn,
    Erro,
}
impl std::fmt::Display for LogType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Info => fmt.write_str("INFO"),
            Self::Warn => fmt.write_str("WARN"),
            Self::Erro => fmt.write_str("ERRO"),
        }
    }
}
pub struct LogFile {
    app_name: String,
    log_dir: String,
    file_date: chrono::Date<chrono::Utc>,
    log_file: std::fs::File,
}
impl LogFile {
    fn chk_dir(dir: &str) -> bool {
        let path = std::path::PathBuf::from(dir);
        if path.is_dir() {
            true
        } else {
            false
        }
    }
    fn make_dir(dir: &str) -> anyhow::Result<()> {
        std::fs::create_dir(dir)?;
        Ok(())
    }
    fn write_line(file: &mut std::fs::File, msg: &str) -> anyhow::Result<()> {
        let message = if msg.ends_with('\n') {
            msg.to_owned()
        } else {
            format!("{}\n", msg)
        };
        Ok(file.write_all(message.as_bytes())?)
    }
    fn roll(
        date: Option<chrono::Date<chrono::Utc>>,
        file: Option<&std::fs::File>,
        app_name: &str,
        log_dir: &str,
    ) -> anyhow::Result<Option<(chrono::Date<chrono::Utc>, std::fs::File)>> {
        let now_date = chrono::Utc::now().date();
        let start_date = date.unwrap_or(now_date.clone());
        if start_date.eq(&now_date) && file.is_some() {
            Ok(None)
        } else {
            let year = format!("{:04}", now_date.year());
            let month = format!("{:02}", now_date.month());
            let day = format!("{:02}", now_date.day());
            let new_file_name = format!(
                "{}_{}-{}-{}.log",
                app_name, year, month, day
            );
            let mut path = std::path::PathBuf::from(log_dir);
            path.push(new_file_name);
            { // lock for creation
                std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)?;
            }
            Ok(Some((
                now_date,
                std::fs::OpenOptions::new()
                    .write(true)
                    .open(path)?
            )))
        }
    }
    fn new(app_name: &str, log_dir: &str) -> anyhow::Result<Self> {
        if !Self::chk_dir(log_dir) {
            Self::make_dir(log_dir)?;
        }
        let (date, file) = Self::roll(
            None, None, app_name, log_dir
        )?.unwrap();
        Ok(
            LogFile {
                app_name: app_name.to_owned(),
                log_dir: log_dir.to_owned(),
                file_date: date,
                log_file: file,
            }
        )
    }
    pub fn from_dir(log_dir: &str, app_name: &str) -> anyhow::Result<Self> {
        Self::new(app_name, log_dir)
    }
    pub fn from_env(log_dir_var: &str, app_name_var: &str) -> anyhow::Result<Self> {
        let log_dir = std::env::var(log_dir_var)?;
        let app_name = std::env::var(app_name_var)?;
        Self::new(&app_name, &log_dir)
    }
    fn log(&mut self, log_type: LogType, msg: &str) -> anyhow::Result<()> {
        match Self::roll(
            Some(self.file_date),
            Some(&self.log_file),
            &self.app_name,
            &self.log_dir
        )? {
            Some((date, file)) => {
                self.file_date = date;
                self.log_file = file;
            },
            None => {},
        }
        let log_msg = format!(
            "{} {} {}",
            chrono::Utc::now().to_rfc3339(),
            log_type,
            msg
        );
        Self::write_line(&mut self.log_file, &log_msg)?;
        Ok(())
    }
    pub fn info(&mut self, msg: &str) -> anyhow::Result<()> {
        self.log(LogType::Info, msg)
    }
    pub fn warn(&mut self, msg: &str) -> anyhow::Result<()> {
        self.log(LogType::Warn, msg)
    }
    pub fn erro(&mut self, msg: &str) -> anyhow::Result<()> {
        self.log(LogType::Erro, msg)
    }
}
#[cfg(test)]
mod tests {
    use super::LogFile;
    #[test]
    fn test() {
        let mut log_file = LogFile::from_dir(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/test_dir",
            ),
            "log_roll_test",
        ).unwrap();
        log_file.info("Info test!").unwrap();
        log_file.warn("Warn test!").unwrap();
        log_file.erro("Erro test!").unwrap();
    }
}
