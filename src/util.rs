use chrono::Duration as Delta;
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use console::Style;
use log::info;
use rusoto_core::{HttpClient, Region};
use rusoto_credential::{AutoRefreshingProvider, ChainProvider, ProfileProvider};
use rusoto_logs::{
    CloudWatchLogs, CloudWatchLogsClient, DescribeLogGroupsRequest, FilterLogEventsRequest,
};
use std::result::Result;
use std::time::Duration;

pub enum AWSResponse {
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

pub fn create_filter_request(
    group: &String,
    start: Duration,
    filter: Option<String>,
    token: Option<String>,
) -> FilterLogEventsRequest {
    let mut req = FilterLogEventsRequest::default();
    let delta = calculate_start_time(Local::now(), start);
    req.start_time = delta;
    req.next_token = token;
    req.limit = Some(100);
    req.filter_pattern = filter;
    req.log_group_name = group.to_string();
    return req;
}

pub fn create_filter_from_timestamp(
    group: &String,
    start: Option<i64>,
    filter: Option<String>,
    token: Option<String>,
) -> FilterLogEventsRequest {
    let mut req = FilterLogEventsRequest::default();
    req.start_time = start;
    req.next_token = token;
    req.limit = Some(100);
    req.filter_pattern = filter;
    req.log_group_name = group.to_string();
    return req;
}

fn print_date(time: Option<i64>) -> String {
    match time {
        //TODO: WTF!!
        Some(x) => DateTime::<Local>::from(DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp(x / 1000, 0),
            Utc,
        ))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string(),
        None => "".to_owned(),
    }
}

pub fn fetch_logs(
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

// TODO: Ugly, make it better please
pub fn list_log_groups(c: &CloudWatchLogsClient) -> Result<(), String> {
    let mut req = DescribeLogGroupsRequest::default();
    loop {
        info!("Sending list log groups request...");
        let resp = c.describe_log_groups(req).sync().unwrap();
        match resp.log_groups {
            Some(x) => {
                for group in x {
                    println!("{}", group.log_group_name.unwrap())
                }
            }
            None => break,
        }
        match resp.next_token {
            Some(x) => {
                req = DescribeLogGroupsRequest::default();
                req.next_token = Some(x)
            }
            None => break,
        }
    }
    Ok(())
}
