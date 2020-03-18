use clap::{App, Arg, ArgMatches};
use ctrlc;
mod util;
use humantime::parse_duration;
use rusoto_core::Region;
use rusoto_logs::CloudWatchLogsClient;
use std::str::FromStr;
use structopt::StructOpt;
use util::{
    client_with_profile, create_filter_from_timestamp, create_filter_request, fetch_logs,
    list_log_groups, AWSResponse,
};

#[derive(Debug, StructOpt)]
#[structopt(name = "awstail", about = "Like tail but for Amazon")]
pub struct CliOptions {
    #[structopt(short, long)]
    list: bool,
    #[structopt(short, long)]
    group: String,
    #[structopt(short, long)]
    watch: bool,
}

fn get_options<'a>() -> ArgMatches<'a> {
    return App::new("awstail")
        .version("0.4.0")
        .author("Yoandy Rodriguez <yoandy.rmartinez@gmail.com>")
        .about("like tail -f for AWS Cloudwatch")
        .arg(
            Arg::with_name("list")
                .short("l")
                .required(true)
                .takes_value(false)
                .help("List existing log groups")
                .conflicts_with_all(&["group", "watch", "since"]),
        )
        .arg(
            Arg::with_name("group")
                .short("g")
                .required(true)
                .takes_value(true)
                .value_name("LOG_GROUP")
                .conflicts_with("list")
                .help("Log group name"),
        )
        .arg(
            Arg::with_name("region")
                .short("r")
                .required(false)
                .value_name("REGION")
                .help("AWS region (defaults to us-east-1)"),
        )
        .arg(
            Arg::with_name("profile")
                .short("p")
                .required(false)
                .value_name("PROFILE")
                .help("Profile if using other than 'default'"),
        )
        .arg(
            Arg::with_name("since")
                .short("s")
                .required(false)
                .value_name("SINCE")
                .help("Take logs since given time (defaults to 5 minutes)"),
        )
        .arg(
            Arg::with_name("watch")
                .short("w")
                .required(false)
                .value_name("WATCH")
                .help("Keep watching for new logs every n seconds (defaults to 10)"),
        )
        .get_matches();
}

fn main() {
    ctrlc::set_handler(move || std::process::exit(0))
        .expect("Could not set Ctrl+C handler...bailing out");
    let matches = get_options();
    let region = match matches.value_of("region") {
        Some(m) => Region::from_str(m),
        None => Ok(Region::UsEast1),
    };
    let client = match matches.value_of("profile") {
        Some(m) => client_with_profile(m, region.unwrap()),
        None => CloudWatchLogsClient::new(region.unwrap()),
    };
    if matches.is_present("list") {
        list_log_groups(&client).unwrap();
    } else {
        let group = matches.value_of("group").unwrap();
        let mtime = match matches.value_of("since") {
            Some(m) => parse_duration(m),
            None => parse_duration("5m"),
        };
        let timeout = match matches.value_of("timeout") {
            Some(m) => parse_duration(m),
            None => parse_duration("30s"),
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
    }
}
