use std::fmt;
use std::io::{self, BufRead};

#[derive(Debug)]
pub enum ReportError {
    IO(String),
}

impl std::error::Error for ReportError {}

impl fmt::Display for ReportError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReportError::IO(e) => write!(f, "IOError: {}", e),
        }
    }
}

impl From<io::Error> for ReportError {
    fn from(error: io::Error) -> Self {
        ReportError::IO(error.to_string())
    }
}

pub fn run() -> Result<(), ReportError> {
    let mut config = String::new();
    let mut sessions = String::new();
    let mut config_done = false;
    for line in io::stdin().lock().lines() {
        let raw_line = line?;
        if raw_line == "" {
            config_done = true;
        } else {
            if !config_done {
                config = format!("{}\n{}", config, raw_line);
            } else {
                sessions = format!("{}{}", sessions, raw_line);
            }
        }
    }
    println!("{}", config);
    println!("{}", sessions);
    Ok(())
}
