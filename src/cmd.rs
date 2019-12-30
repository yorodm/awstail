use chrono::Duration as Delta;
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use clap::ArgMatches;
use console::Style;
use humantime::parse_duration;
use rusoto_core::HttpClient;
use rusoto_core::Region;
use rusoto_credential::{AutoRefreshingProvider, ChainProvider, ProfileProvider};
use rusoto_logs::{CloudWatchLogs, CloudWatchLogsClient, FilterLogEventsRequest};
use std::result::Result;
use std::str::FromStr;
use std::time::Duration;

enum AWSResponse {
    Token(String),
    LastLog(Option<i64>),
}

fn calculate_start_time(from: DateTime<Local>, delta: Duration) -> Option<i64> {
    let chrono_delta = Delta::from_std(delta).unwrap();
    let start_time = from.checked_sub_signed(chrono_delta).unwrap();
    // Amazon uses time in UTC so we have to convert
    let utc_time = DateTime::<Utc>::from_utc(start_time.naive_utc(), Utc);
    return Some(utc_time.timestamp_millis());
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
    req.limit = Some(100);
    req.log_group_name = group.to_string();
    return req;
}

fn create_filter_from_timestamp(
    group: &str,
    start: Option<i64>,
    token: Option<String>,
) -> FilterLogEventsRequest {
    let mut req = FilterLogEventsRequest::default();
    req.start_time = start;
    req.next_token = token;
    req.limit = Some(100);
    req.log_group_name = group.to_string();
    return req;
}

fn print_date(time: Option<i64>) -> String {
    match time {
        Some(x) => DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(x / 1000, 0), Utc)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
        None => "".to_owned(),
    }
}

fn fetch_logs(
    client: &CloudWatchLogsClient,
    req: FilterLogEventsRequest,
    timeout: Duration,
) -> AWSResponse {
    let response = client
        .filter_log_events(req.clone()) // we may need this later
        .with_timeout(timeout)
        .sync()
        .unwrap();
    let events = response.events.unwrap();
    let green = Style::new().green();
    let mut last: Option<i64> = None;
    for event in events {
        println!(
            "{} {}",
            green.apply_to(print_date(event.timestamp)),
            event.message.unwrap().trim(),
        );
        last = event.timestamp
    }
    match response.next_token {
        Some(x) => AWSResponse::Token(x),
        None => match last {
            Some(t) => AWSResponse::LastLog(Some(t)),
            None => AWSResponse::LastLog(req.start_time),
        },
    }
}

pub fn client_with_profile(name: &str, region: Region) -> CloudWatchLogsClient {
    let mut profile = ProfileProvider::new().unwrap();
    profile.set_profile(name);
    let chain = ChainProvider::with_profile_provider(profile);
    let credentials = AutoRefreshingProvider::<ChainProvider>::new(chain).unwrap();
    CloudWatchLogsClient::new_with(HttpClient::new().unwrap(), credentials, region)
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
            (local - Duration::days(1)).timestamp_millis()
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
    let refetch = match matches.value_of("fetch") {
        Some(m) => parse_duration(m),
        None => parse_duration("10s"),
    };
    let client = match matches.value_of("profile") {
        Some(m) => client_with_profile(m, region.unwrap()),
        None => CloudWatchLogsClient::new(region.unwrap()),
    };
    let sleep_for = match matches.value_of("watch") {
        Some(m) => parse_duration(m),
        None => parse_duration("10s"),
    };
    let mut token: Option<String> = None;
    let mut req = create_filter_request(group, mtime.unwrap(), token);
    loop {
        match fetch_logs(&client, req, timeout.unwrap()) {
            AWSResponse::Token(x) => {
                token = Some(x);
                req = create_filter_request(group, mtime.unwrap(), token);
            }
            AWSResponse::LastLog(t) => {
                token = None;
                req = create_filter_from_timestamp(group, t, token);
                std::thread::sleep(sleep_for.unwrap());
            }
        };
    }
    Ok(())
}
