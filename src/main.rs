use clap::{App, Arg};
use cmd::run;
use ctrlc;
mod cmd;

fn main() {
    ctrlc::set_handler(move || std::process::exit(0))
        .expect("Could not set Ctrl+C handler...bailing out");
    let matches = App::new("awstail")
        .version("0.2.0")
        .author("Yoandy Rodriguez <yoandy.rmartinez@gmail.com>")
        .about("like tail -f for AWS Cloudwatch")
        .arg(
            Arg::with_name("group")
                .required(true)
                .takes_value(true)
                .value_name("LOG_GROUP")
                .help("Log group name")
                .index(1),
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
        std::process::exit(1);
    }
}
