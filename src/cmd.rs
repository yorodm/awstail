use chrono::Duration as Delta;
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use clap::ArgMatches;
use humantime::parse_duration;
use rusoto_core::Region;
use rusoto_logs::{CloudWatchLogs, CloudWatchLogsClient, FilterLogEventsRequest};
use std::result::Result;
use std::str::FromStr;
use std::time::Duration;

fn calculate_start_time(from: DateTime<Local>, delta: Duration) -> Option<i64> {
    let chrono_delta = Delta::from_std(delta).unwrap();
    let start_time = from.checked_sub_signed(chrono_delta).unwrap();
    // Amazon uses time in UTC so we have to convert
    let utc_time = DateTime::<Utc>::from_utc(start_time.naive_utc(), Utc);
    return Some(utc_time.timestamp() * 1000);
}

fn create_filter_request(
    group: &str,
    start: Duration,
    token: Option<String>,
) -> FilterLogEventsRequest {
    let mut req = FilterLogEventsRequest::default();
    let delta = calculate_start_time(Local::now(), start);
    req.start_time = delta;
    req.next_token = token;
    req.log_group_name = group.to_string();
    return req;
}

fn print_date(time: Option<i64>) -> String {
    match time {
        Some(x) => NaiveDateTime::from_timestamp(x / 1000, 0)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
        None => "".to_owned(),
    }
}

fn fetch_logs(
    client: &CloudWatchLogsClient,
    req: FilterLogEventsRequest,
    timeout: Duration,
) -> Option<String> {
    let logs = client.filter_log_events(req).sync().unwrap();
    let events = logs.events.unwrap();
    for event in events.into_iter() {
        println!(
            "{} {} {}",
            print_date(event.timestamp),
            event.message.unwrap().trim(),
            Local::now().to_rfc3339()
        );
    }
    return logs.next_token;
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Local};
    use humantime::parse_duration;

    #[test]
    fn test_calculate_start_time() {
        let local = Local::now();
        let duration = parse_duration("1d").unwrap();
        assert_eq!(
            calculate_start_time(local, duration).unwrap(),
            (local - Duration::days(1)).timestamp() * 1000
        )
    }
}

pub fn run(matches: ArgMatches) -> Result<(), String> {
    let group = matches.value_of("group").unwrap();
    let mtime = match matches.value_of("since") {
        Some(m) => parse_duration(m),
        None => parse_duration("5m"),
    };
    let timeout = match matches.value_of("timeout") {
        Some(m) => parse_duration(m),
        None => parse_duration("30s"),
    };
    let region = match matches.value_of("region") {
        Some(m) => Region::from_str(m),
        None => Ok(Region::UsEast1),
    };
    let mut token: Option<String> = None;
    let mut req = create_filter_request(group, mtime.unwrap(), token);
    let client = CloudWatchLogsClient::new(region.unwrap());
    loop {
        match fetch_logs(&client, req, timeout.unwrap()) {
            Some(x) => token = Some(x),
            None => break,
        };
        req = create_filter_request(group, mtime.unwrap(), token);
    }
    Ok(())
}
