use ctrlc;
mod util;
use humantime::parse_duration;
use rusoto_core::Region;
use std::str::FromStr;
use structopt::StructOpt;
use util::{
    client_with_profile, create_filter_from_timestamp, create_filter_request, fetch_logs,
    list_log_groups, AWSResponse,
};

#[derive(Debug, StructOpt)]
#[structopt(name = "awstail", about = "Like tail but for Amazon")]
pub struct CliOptions {
    #[structopt(short, long,conflicts_with_all(&["group", "watch", "since", "filter"]))]
    list: bool,
    #[structopt(short, long)]
    group: Option<String>,
    #[structopt(short, long, default_value = "30s")]
    watch: String,
    #[structopt(short, long, default_value = "5min")]
    since: String,
    #[structopt(short, long)]
    filter: Option<String>,
    #[structopt(short, long, default_value = "us-east-1")]
    region: String,
    #[structopt(short, long, default_value = "default")]
    profile: String,
    #[structopt(short, long, default_value = "30s")]
    timeout: String,
}

fn main() {
    ctrlc::set_handler(move || std::process::exit(0))
        .expect("Could not set Ctrl+C handler...bailing out");
    let matches = CliOptions::from_args();
    let region = Region::from_str(&matches.region).unwrap();
    let client = client_with_profile(&matches.profile, region);
    if matches.list {
        list_log_groups(&client).unwrap();
    } else {
        let group = matches.group.unwrap();
        let mtime = parse_duration(&matches.since);
        let timeout = parse_duration(&matches.timeout);
        let sleep_for = parse_duration(&matches.watch);
        let filter = matches.filter;
        let mut token: Option<String> = None;
        let mut req = create_filter_request(&group, mtime.unwrap(), filter.clone(), token);
        loop {
            match fetch_logs(&client, req, timeout.unwrap()) {
                AWSResponse::Token(x) => {
                    token = Some(x);
                    req = create_filter_request(&group, mtime.unwrap(), filter.clone(), token);
                }
                AWSResponse::LastLog(t) => {
                    token = None;
                    req = create_filter_from_timestamp(&group, t, filter.clone(), token);
                    std::thread::sleep(sleep_for.unwrap());
                }
            };
        }
    }
}
