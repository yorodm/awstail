use ctrlc;
mod funcs;
use funcs::{
    client_with_profile, create_filter_from_timestamp, create_filter_request, fetch_logs,
    list_log_groups, AWSResponse,
};
use gumdrop::Options;
use humantime::parse_duration;
use log::info;
use rusoto_core::Region;
use std::str::FromStr;


#[derive(Debug, Options, PartialEq)]
/// Tail the cloud
pub struct CliOptions {
    /// prints help message
    #[options(help = "print help message")]
    pub(crate) help: bool,
    /// region
    #[options(help = "change default region")]
    pub(crate) region: Option<String>,
    /// profile
    #[options(help = "change default profile")]
    pub(crate) profile: Option<String>,
    #[options(command)]
    pub(crate) commands: Option<CommandOptions>,
}

#[derive(Debug, Options, PartialEq)]
pub enum CommandOptions {
    #[options(help = "list existing log groups")]
    List(ListOpts),
    #[options(help = "access existing logs from a group")]
    Logs(LogsOptions),
	#[options(help = "Show program version")]
	Version(Version),
}

#[derive(Debug, Options, PartialEq)]
pub struct Version{}

#[derive(Debug, Options, PartialEq)]
/// List existing log groups
pub struct ListOpts {
    #[options(help = "print help message")]
    pub(crate) help: bool,
}

#[derive(Debug, Options, PartialEq)]
/// Fetch logs from groups
pub struct LogsOptions {
    #[options(help = "print help message")]
    pub(crate) help: bool,
    /// group name
    #[options(help = "group name")]
    pub(crate) group: Option<String>,
    /// keep watching logs (like tail -f) refresh after a given time
    #[options(help = "keep watching logs and refresh after a given time")]
    pub(crate) watch: Option<String>,
    /// fetch logs starting a given time period
    #[options(help = "fetch logs starting a given time period")]
    pub(crate) since: Option<String>,
    /// filter logs
    #[options(help = "filter results given a pattern")]
    pub(crate) filter: Option<String>,
    /// timeout after a given time period
    #[options(help = "timeout period")]
    pub(crate) timeout: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    ctrlc::set_handler(move || std::process::exit(0))
        .expect("Could not set Ctrl+C handler...bailing out");
    let matches: CliOptions = CliOptions::parse_args_default_or_exit();
    let region = matches
        .region
        .map_or(Region::default(), |x| Region::from_str(&x).unwrap());
	env_logger::init();
    let profile = matches.profile.map_or("default".to_owned(), |x| x);
    let client = client_with_profile(&profile, region);
    if let Some(commands) = matches.commands {
        match commands {
            CommandOptions::List(_) => list_log_groups(&client).await?,
            CommandOptions::Logs(g) => {
                let group = g.group.expect("A group name must be provided");
                let mtime = g
                    .since
                    .map_or(parse_duration("5min"), |x| parse_duration(&x))?;
                let timeout = g
                    .timeout
                    .map_or(parse_duration("1min"), |x| parse_duration(&x))?;
                let sleep_for = g.watch.and_then(|x| parse_duration(&x).ok());
                let filter = g.filter;
                let mut token: Option<String> = None;
                let mut req = create_filter_request(&group, mtime, filter.clone(), token);
                loop {
                    match fetch_logs(&client, req, timeout).await? {
                        AWSResponse::Token(x) => {
							info!("Got a Token response");
                            token = Some(x);
                            req = create_filter_request(&group, mtime, filter.clone(), token);
                        }
                        AWSResponse::LastLog(t) => match sleep_for {
                            Some(x) => {
								info!("Got a lastlog response");
                                token = None;
                                req =
                                    create_filter_from_timestamp(&group, t, filter.clone(), token);
                                info!("Waiting {:?} before requesting logs again...", x);
								tokio::time::delay_for(x).await
                            }
                            None => break,
                        },
                    };
                }
            }
            CommandOptions::Version(_) => {
				println!("awstail {}", env!("CARGO_PKG_VERSION"));
			}
        }
    };
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_logs_command() {
        let _ = CliOptions::parse_args_default(&vec![
            "logs",
            "-g /aws/lambda/Pepe",
            "-f \"Pattern\"",
            "-w 30s",
        ])
        .unwrap();
    }
}
