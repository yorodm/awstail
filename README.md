# AWSTail

[AWSTail](https://gitbuh.com/yorodm/awstail) is a tail like tool for [AWS Cloudwatch](https://aws.amazon.com/cloudwatch/).

This is a simple tool that allows you to monitor your [AWS Cloudwatch](https://aws.amazon.com/cloudwatch/) logs
in a way similar to `tail -f`.

To install just download the binaries from the release page or clone this repo and build it yourself.

```sh
Usage: target\debug\awstail.exe [OPTIONS]

awstail: Tail log streams from AWS Cloudwatch logs


Optional arguments:
  -h, --help             show this message
  -v, --verbose VERBOSE  be verbose
  -r, --region REGION    region
  -p, --profile PROFILE  profile

Available commands:
  list  List existing groups
  logs  Fetch logs from groups
```
