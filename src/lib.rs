use chrono::prelude::*;
use serde::Deserialize;
use std::cmp::Ordering;
use std::collections::HashMap;
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

mod my_optional_date_format {
    use chrono::{DateTime, Local, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer};

    const FORMAT: &str = "%Y%m%dT%H%M%SZ";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Local>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Some(
            Utc.datetime_from_str(&s, FORMAT)
                .map_err(serde::de::Error::custom)?
                .with_timezone(&Local),
        ))
    }
}

#[derive(Debug)]
pub struct TimewarriorData {
    pub config: HashMap<String, String>,
    pub sessions: Vec<Session>,
}

impl TimewarriorData {
    pub fn from_std() -> Result<Self, ReportError> {
        let mut input_string = String::new();
        for line in io::stdin().lock().lines() {
            input_string = format!("{}\n{}", input_string, line?);
        }
        Self::from_string(input_string.trim().into())
    }

    pub fn from_string(input: String) -> Result<Self, ReportError> {
        let input_vec = &input.split("\n\n").collect::<Vec<&str>>();
        let mut config = HashMap::new();
        for line in input_vec[0].lines() {
            let setting = line.split(": ").collect::<Vec<&str>>();
            config.insert(setting[0].into(), setting[1].into());
        }
        Ok(TimewarriorData {
            config,
            sessions: Session::from_json(&input_vec[1])?,
        })
    }
}

#[derive(Debug, Deserialize, Eq)]
pub struct Session {
    pub id: usize,
    #[serde(with = "my_date_format")]
    pub start: DateTime<Local>,
    #[serde(default)]
    #[serde(with = "my_optional_date_format")]
    pub end: Option<DateTime<Local>>,
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

pub fn run() -> Result<(), ReportError> {
    let data = TimewarriorData::from_std()?;
    dbg!(data);
    Ok(())
}
