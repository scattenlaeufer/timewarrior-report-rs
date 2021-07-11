use chrono::prelude::*;
use serde::Deserialize;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::io::{self, BufRead};

/// An enum to represent errors occurring while processing report data from Timewarrior
#[derive(Debug)]
pub enum ReportError {
    /// An error, which occurred while parsing data from standard in
    IO(String),
    /// An error, which occurred while deserializing or serializing a session from JSON
    SerdeJson(String),
    /// Some other error
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

/// A representation of the data within the report
#[derive(Debug, Eq)]
pub struct TimewarriorData {
    /// The configurations passed to the report
    pub config: HashMap<String, String>,
    /// A vector of all tracked sessions within the report
    pub sessions: Vec<Session>,
}

impl PartialEq for TimewarriorData {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config && self.sessions == other.sessions
    }
}

impl TimewarriorData {
    /// Read the report from standard input
    ///
    /// This should be the usual way to read the report data.
    pub fn from_stdin() -> Result<Self, ReportError> {
        let mut input_string = String::new();
        for line in io::stdin().lock().lines() {
            input_string = format!("{}\n{}", input_string, line?);
        }
        Self::from_string(input_string.trim().into())
    }

    /// Read the report from a given string
    ///
    /// # Example
    ///
    /// ```rust
    /// use timewarrior_report::TimewarriorData;
    ///
    /// let report_data = TimewarriorData::from_string("test: test\n\n[]".into()).unwrap();
    /// assert_eq!(
    ///     report_data,
    ///     TimewarriorData {
    ///         config: [("test".to_string(), "test".to_string())]
    ///             .iter()
    ///             .cloned()
    ///             .collect(),
    ///         sessions: Vec::new(),
    ///     }
    /// );
    /// ```
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
/// A tracked session from Timewarrior
#[derive(Debug, Deserialize, Eq)]
pub struct Session {
    /// ID of the session within Timewarrior
    pub id: usize,
    /// Start time of the session
    #[serde(with = "my_date_format")]
    pub start: DateTime<Local>,
    /// End time of the session. `Some(DateTime<Local>)` if it did end, `None` otherwise.
    #[serde(default)]
    #[serde(with = "my_optional_date_format")]
    pub end: Option<DateTime<Local>>,
    /// Tags attached to the session
    pub tags: Vec<String>,
    /// Annotation of the session. `Some(String)` if the session has an annotation, `None`
    /// otherwise.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_simple_timewarrior_data() {
        let report_data = TimewarriorData::from_string("test: test\n\n[]".into()).unwrap();
        assert_eq!(
            report_data,
            TimewarriorData {
                config: [("test".to_string(), "test".to_string())]
                    .iter()
                    .cloned()
                    .collect(),
                sessions: Vec::new(),
            }
        );
    }

    #[test]
    fn create_session_without_end_date() {
        let test_session = serde_json::from_str::<Session>(
            "{\"id\":1,\"start\":\"20210711T103400Z\",\"tags\":[\"test\"],\"annotation\":\"this is a test\"}",
        )
        .unwrap();
        assert_eq!(
            test_session,
            Session {
                id: 1,
                start: DateTime::<Utc>::from_utc(
                    NaiveDate::from_ymd(2021, 07, 11).and_hms(10, 34, 00),
                    Utc
                )
                .with_timezone(&Local),
                end: None,
                tags: vec!["test".to_string()],
                annotation: Some("this is a test".to_string()),
            }
        );
    }

    #[test]
    fn create_session_with_end_date() {
        let test_session = serde_json::from_str::<Session>(
            "{\"id\":1,\"start\":\"20210711T103400Z\",\"end\":\"20210711T113400Z\",\"tags\":[\"test\"],\"annotation\":\"this is a test\"}",
        )
        .unwrap();
        assert_eq!(
            test_session,
            Session {
                id: 1,
                start: DateTime::<Utc>::from_utc(
                    NaiveDate::from_ymd(2021, 07, 11).and_hms(10, 34, 00),
                    Utc
                )
                .with_timezone(&Local),
                end: Some(
                    DateTime::<Utc>::from_utc(
                        NaiveDate::from_ymd(2021, 07, 11).and_hms(11, 34, 00),
                        Utc
                    )
                    .with_timezone(&Local)
                ),
                tags: vec!["test".to_string()],
                annotation: Some("this is a test".to_string()),
            }
        );
    }
}
