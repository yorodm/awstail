extern crate clap;
extern crate chrono;
extern crate humantime;
extern crate rusoto_core;
extern crate rusoto_logs;

use clap::{Arg, App};
use cmd::run;

mod cmd;
// fn run(matches: ArgMatches)

// }

fn main() {
    let matches = App::new("awstail")
        .version("0.1.0")
        .author("Yoandy Rodriguez <yoandy.rmartinez@gmail.com>")
        .about("like tail -f for AWS Cloudwatch")
        .arg(Arg::with_name("group")
             .short("g")
             .required(true)
             .takes_value(true)
             .value_name("LOG_GROUP")
             .help("Log group name"))
        .arg(Arg::with_name("region")
             .short("r")
             .required(false)
             .value_name("REGION")
             .help("AWS region (defaults to us-east-1)"))
        .arg(Arg::with_name("since")
             .short("s")
             .required(false)
             .value_name("SINCE")
             .help("Take logs since given time (defaults to 5 minutes)"))
        .get_matches();
    if let Err(_e) = run(matches) {
        println!("Tuvimos un error");
        std::process::exit(1);
    }
}
