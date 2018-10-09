use chrono::Duration as Delta;
use chrono::{DateTime, Local, Utc};
use clap::ArgMatches;
use humantime::parse_duration;
use rusoto_core::Region;
use rusoto_logs::{CloudWatchLogs, CloudWatchLogsClient, FilterLogEventsRequest};
use std::result::Result;
use std::str::FromStr;
use std::time::Duration;

fn calculate_start_time(delta: Duration) -> Option<i64> {
    let chrono_delta = Delta::from_std(delta).unwrap();
    let start_time = Local::now().checked_sub_signed(chrono_delta).unwrap();
    // Amazon uses time in UTC so we have to convert
    let utc_time = DateTime::<Utc>::from_utc(start_time.naive_utc(), Utc);
    return Some(utc_time.timestamp());
}

//[#test]


fn create_filter_request(
    group: &str,
    start: Option<i64>,
    token: Option<String>,
) -> FilterLogEventsRequest {
    let mut req = FilterLogEventsRequest::default();
    req.start_time = start;
    req.next_token = token;
    req.log_group_name = group.to_string();
    return req;
}

fn fetch_logs(region: Region, group: &str, since: Duration) -> Result<(), String> {
    let client = CloudWatchLogsClient::new(region);
    let delta = calculate_start_time(since);
    let req = create_filter_request(group, delta, None);
    // right now
    let logs = client.filter_log_events(req).sync().unwrap();
    let events = logs.events.unwrap();
    for event in events.into_iter() {
        println!(
            "{} {} {}",
            event.event_id.unwrap(),
            event.timestamp.unwrap(),
            event.message.unwrap()
        );
    }
    return Ok(());
}

pub fn run(matches: ArgMatches) -> Result<(), String> {
    let group = matches.value_of("group").unwrap();
    let mtime = match matches.value_of("since") {
        Some(m) => parse_duration(m),
        None => parse_duration("5m"),
    };
    let region = match matches.value_of("region") {
        Some(m) => Region::from_str(m),
        None => Ok(Region::UsEast1),
    };
    return fetch_logs(region.unwrap(), group, mtime.unwrap());
}
