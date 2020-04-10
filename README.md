# AWSTail

[AWSTail](https://gitbuh.com/yorodm/awstail) is a tail like tool for [AWS Cloudwatch](https://aws.amazon.com/cloudwatch/).

This is a simple tool that allows you to monitor your [AWS Cloudwatch](https://aws.amazon.com/cloudwatch/) logs
in a way similar to `tail -f`.

To install just download the binaries from the release page or clone this repo and build it yourself.

```sh
awstail 0.5.0
Like tail but for Amazon

USAGE:
awstail.exe [FLAGS] [OPTIONS]

FLAGS:
-h, --help Prints help information
-l, --list
-V, --versionPrints version information

OPTIONS:
-f, --filter <filter>
-g, --group <group>
-p, --profile <profile> [default: default]
-r, --region <region> [default: us-east-1]
-s, --since <since> [default: 5min]
-t, --timeout <timeout> [default: 30s]
-w, --watch <watch> [default: 30s]
```
