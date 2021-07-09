use chrono::prelude::*;
use serde::Deserialize;
use std::cmp::Ordering;
use std::fmt;
use std::io::{self, BufRead};

#[derive(Debug)]
pub enum ReportError {
    IO(String),
    SerdeJson(String),
    Other(String),
}

impl std::error::Error for ReportError {}

impl fmt::Display for ReportError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReportError::IO(e) => write!(f, "IOError: {}", e),
            ReportError::SerdeJson(e) => write!(f, "SerdeJsonError: {}", e),
            ReportError::Other(e) => write!(f, "Other Error: {}", e),
        }
    }
}

impl From<io::Error> for ReportError {
    fn from(error: io::Error) -> Self {
        ReportError::IO(error.to_string())
    }
}

impl From<serde_json::Error> for ReportError {
    fn from(error: serde_json::Error) -> Self {
        ReportError::SerdeJson(error.to_string())
    }
}

mod my_date_format {
    use chrono::{DateTime, Local, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer};

    const FORMAT: &str = "%Y%m%dT%H%M%SZ";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Utc
            .datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)?
            .with_timezone(&Local))
    }
}

fn default_end_time() -> DateTime<Local> {
    Local::now()
}

#[derive(Debug)]
pub struct TimewarriorData {
    // TODO: Make this a HashMap to be actually useful
    pub config: String,
    pub sessions: Vec<Session>,
}

#[derive(Debug, Deserialize, Eq)]
pub struct Session {
    pub id: usize,
    #[serde(with = "my_date_format")]
    pub start: DateTime<Local>,
    #[serde(with = "my_date_format")]
    #[serde(default = "default_end_time")]
    pub end: DateTime<Local>,
    pub tags: Vec<String>,
    pub annotation: Option<String>,
}

impl PartialEq for Session {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start
            && self.end == other.end
            && self.id == other.id
            && self.tags == other.tags
            && self.annotation == other.annotation
    }
}

impl Ord for Session {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for Session {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Session {
    fn from_json(data: &str) -> Result<Vec<Session>, ReportError> {
        Ok(serde_json::from_str::<Vec<Session>>(data)?)
    }
}

pub fn get_data() -> Result<TimewarriorData, ReportError> {
    let mut config = String::new();
    let mut sessions_raw = String::new();
    let mut config_done = false;
    for line in io::stdin().lock().lines() {
        let raw_line = line?;
        if raw_line.is_empty() {
            config_done = true;
        } else if !config_done {
            config = format!("{}\n{}", config, raw_line);
        } else {
            sessions_raw = format!("{}{}", sessions_raw, raw_line);
        }
    }

    Ok(TimewarriorData {
        config,
        sessions: Session::from_json(&sessions_raw)?,
    })
}

pub fn run() -> Result<(), ReportError> {
    let data = get_data()?;
    dbg!(data);
    Ok(())
}
