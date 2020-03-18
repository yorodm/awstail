# AWSTail

[AWSTail](https://gitbuh.com/yorodm/awstail) is a tail like tool for [AWS Cloudwatch](https://aws.amazon.com/cloudwatch/).

This is a simple tool that allows you to monitor your [AWS Cloudwatch](https://aws.amazon.com/cloudwatch/) logs
in a way similar to `tail -f`.

To install just download the binaries from the release page or clone this repo and build it yourself.

```sh
awstail 0.4.0
Yoandy Rodriguez <yoandy.rmartinez@gmail.com>
like tail -f for AWS Cloudwatch

USAGE:
    awstail.exe [FLAGS] [OPTIONS] -g <LOG_GROUP> -l

FLAGS:
    -h, --help       Prints help information
    -l               List existing log groups
    -V, --version    Prints version information

OPTIONS:
    -g <LOG_GROUP>        Log group name
    -p <PROFILE>          Profile if using other than 'default'
    -r <REGION>           AWS region (defaults to us-east-1)
    -s <SINCE>            Take logs since given time (defaults to 5 minutes)
    -w <WATCH>            Keep watching for new logs every n seconds (defaults to 10)
```
