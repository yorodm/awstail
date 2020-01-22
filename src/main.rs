use clap::{App, Arg};
use cmd::run;
use ctrlc;
mod cmd;

fn main() {
    ctrlc::set_handler(move || std::process::exit(0))
        .expect("Could not set Ctrl+C handler...bailing out");
    let matches = App::new("awstail")
        .version("0.3.0")
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
    if let Err(_e) = run(matches) {
        std::process::exit(0);
    }
}
