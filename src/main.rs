use ctrlc;
mod funcs;
use funcs::{
    client_with_profile, create_filter_from_timestamp, create_filter_request, fetch_logs,
    list_log_groups, AWSResponse,
};
use humantime::parse_duration;
use log::info;
use rusoto_core::Region;
use gumdrop::Options;
use std::str::FromStr;

#[derive(Debug, Options)]
/// awstail: Tail log streams from AWS Cloudwatch logs
pub struct CliOptions {
	/// show this message
	help: bool,
	/// be verbose
    verbose: usize,
	#[options(command)]
	commands: Option<CommandOptions>,
	/// region
    region: Option<String>,
    /// profile
    profile: Option<String>,
}

#[derive(Debug,Options)]
pub enum CommandOptions {
	/// List existing groups
	List(ListOpts),
	/// Fetch logs from groups
	Logs(LogsOptions)
}

#[derive(Debug,Options)]
pub struct ListOpts {
	help: bool
}

#[derive(Debug, Options)]
pub struct LogsOptions {
	help: bool,
    /// group name
    group: Option<String>,
    /// keep watching logs (like tail -f) refresh after a given time
    watch: Option<String>,
    /// fetch logs starting a given time period
    since: Option<String>,
    /// filter logs
    filter: Option<String>,
    /// timeout after a given time period
    timeout: Option<String>,

}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error>{
    ctrlc::set_handler(move || std::process::exit(0))
        .expect("Could not set Ctrl+C handler...bailing out");
    let matches: CliOptions = CliOptions::parse_args_default_or_exit();
    let region =  matches.region.map_or(Region::default(), |x| Region::from_str(&x).unwrap());
    let profile = matches.profile.map_or("default".to_owned(), |x| x);
    let client = client_with_profile(&profile, region);
    stderrlog::new().module(module_path!()).init().unwrap();
	if let Some(commands) = matches.commands {
		match commands {
		CommandOptions::List(_)=> list_log_groups(&client).await?,
		CommandOptions::Logs(g) => {
        let group = g.group.unwrap();
        let mtime = g.since.map_or(parse_duration("5min"), |x| parse_duration(&x))?;
        let timeout = g.timeout.map_or(parse_duration("1min"), |x| parse_duration(&x))?;
        let sleep_for = g.watch.map(|x| parse_duration(&x));
        let filter = g.filter;
        let mut token: Option<String> = None;
        let mut req = create_filter_request(&group, mtime, filter.clone(), token);
        loop {
            match fetch_logs(&client, req, timeout).await? {
                AWSResponse::Token(x) => {
                    token = Some(x);
                    req = create_filter_request(&group, mtime, filter.clone(), token);
                }
                AWSResponse::LastLog(t) => match sleep_for {
                    Some(x) => {
                        token = None;
                        req = create_filter_from_timestamp(&group, t, filter.clone(), token);
                        info!("Waiting {:?} before requesting logs again...", x.unwrap());
                        std::thread::sleep(x.unwrap());
                    }
                    None => break,
                },
            };
        }
		}
	}
	};
	Ok(())
}
