use chrono::{DateTime, Utc};
use std::time::SystemTime;

pub fn parse_system_time(input: &str) -> SystemTime {
    let dt: DateTime<Utc> = DateTime::parse_from_rfc3339(input)
        .unwrap()
        .with_timezone(&Utc);
    let system_time: SystemTime = dt.into();
    return system_time;
}
