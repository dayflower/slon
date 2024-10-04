# slon - Slack opinionated notifier

Inspired by [nagomiso/slackutils](https://github.com/nagomiso/slackutils).

## CLI

```
Usage: slon [OPTIONS]

Options:
  -e, --endpoint <ENDPOINT>      Slack API endpoint or webhook URL [default: https://slack.com/api/chat.postMessage]
  -c, --channel <CHANNEL>        Target channel
  -t, --header <HEADER>          Message title
  -b, --footer <FOOTER>          Message footer
  -m, --message <MESSAGE>        Message body
  -f, --field [<FIELD>...]       Message fields
  -r, --color <COLOR>            Message color
  -u, --username <USERNAME>      Sender user name
  -i, --icon-emoji <ICON_EMOJI>  Sender icon emoji
  -v, --verbose                  Verbose output
  -h, --help                     Print help
  -V, --version                  Print version
```

Almost all options are optional, as long as at least one of the following is supplied: the header, the footer, the message, or the fields.

To specify the Slack API token, set the `SLACK_TOKEN` environment variable. You can omit the token when sending messages via a Slack Incoming Webhook.
