use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};

pub fn time_to_human_readable(naive_datetime: NaiveDateTime) -> String {
    // Parse the ISO 8601 timestamp
    // let naive_datetime = NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S%.f").unwrap();

    // Convert to a DateTime<Utc> for formatting
    let datetime: DateTime<Utc> = TimeZone::from_utc_datetime(&Utc, &naive_datetime);

    // Format the timestamp into a more readable format
    datetime.format("%d/%m/%Y %H:%M:%S").to_string()
}
