use std::{collections::HashMap, env};

use clap::Parser;
use reqwest::{blocking::Client, header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Text {
    #[allow(dead_code)]
    #[serde(rename = "plain_text")]
    PlainText {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        emoji: Option<bool>,
    },

    #[serde(rename = "mrkdwn")]
    Markdown {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        verbatim: Option<bool>,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Block {
    #[serde(rename = "header")]
    Header { text: Text },

    #[serde(rename = "section")]
    TextSection { text: Text },

    #[serde(rename = "section")]
    TextFieldsSection { fields: Vec<Text> },

    #[serde(rename = "context")]
    Context { elements: Vec<Text> },
}

#[derive(Debug, Serialize)]
struct Attachment {
    blocks: Vec<Block>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
}

#[derive(Debug, Serialize)]
struct ChatPostMessageRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    channel: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    icon_emoji: Option<String>,

    attachments: Vec<Attachment>,
}

#[derive(Deserialize)]
struct SlackResponse {
    ok: bool,
    error: Option<String>,
}

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Slack opinionated notifier",
    version = "0.1.0",
    long_about
)]
struct Args {
    #[arg(
        long,
        short = 'e',
        default_value = "https://slack.com/api/chat.postMessage",
        help = "Slack API endpoint or webhook URL"
    )]
    endpoint: Option<String>,

    #[arg(long, short = 'c', help = "Target channel")]
    channel: Option<String>,

    #[arg(long, short = 't', help = "Message title")]
    header: Option<String>,

    #[arg(long, short = 'b', help = "Message footer")]
    footer: Option<String>,

    #[arg(long, short = 'm', help = "Message body")]
    message: Option<String>,

    #[arg(long, short='f', num_args(0..), help = "Message fields")]
    field: Vec<String>,

    #[arg(long, short = 'r', help = "Message color")]
    color: Option<String>,

    #[arg(long, short = 'u', help = "Sender user name")]
    username: Option<String>,

    #[arg(long, short = 'i', help = "Sender icon emoji")]
    icon_emoji: Option<String>,

    #[arg(long, short = 'v', help = "Verbose output")]
    verbose: bool,
}

fn main() {
    let colors: HashMap<_, _> = HashMap::from([
        // official colors
        ("good", "#2eb886"),
        ("warning", "#daa038"),
        ("danger", "#a30100"),
        // unofficial colors
        ("success", "#2eb886"),
        ("error", "#a30100"),
        ("info", "#3aa3e3"),
        ("black", "#1e2d2f"),
    ]);

    let args = Args::parse();

    let token = env::var("SLACK_TOKEN");

    let mut blocks: Vec<Block> = vec![];

    if let Some(header) = args.header {
        blocks.push(Block::Header {
            text: Text::PlainText {
                text: header,
                emoji: Some(true),
            },
        });
    }

    if let Some(message) = args.message {
        blocks.push(Block::TextSection {
            text: Text::Markdown {
                text: message,
                verbatim: None,
            },
        })
    }

    if !args.field.is_empty() {
        blocks.push(Block::TextFieldsSection {
            fields: args
                .field
                .iter()
                .map(|text| Text::Markdown {
                    text: text.to_string(),
                    verbatim: None,
                })
                .collect(),
        });
    }

    if let Some(footer) = args.footer {
        blocks.push(Block::Context {
            elements: vec![Text::Markdown {
                text: footer,
                verbatim: None,
            }],
        });
    }

    if blocks.is_empty() {
        eprintln!("ERROR: At least one of Header, Footer, Message and Fields is required");
        std::process::exit(1);
    }

    let color = args
        .color
        .as_ref()
        .and_then(|k| colors.get(k.as_str()).map(|v| v.to_string()))
        .or(args.color.clone());

    let req: ChatPostMessageRequest = ChatPostMessageRequest {
        channel: args.channel,
        username: args.username,
        icon_emoji: args.icon_emoji,
        attachments: vec![Attachment {
            blocks: blocks,
            color: color,
        }],
    };

    if args.verbose {
        if let Ok(request_body) = serde_json::to_string_pretty(&req) {
            println!("{}", request_body);
        }
    }

    let client = Client::builder()
        .user_agent("slon/0.1.0")
        .build()
        .expect("Failed to create HTTP client");

    let mut request_builder = client
        .post(args.endpoint.expect("Endpoint is required"))
        .header(header::CONTENT_TYPE, "application/json");

    if let Ok(token) = token {
        request_builder =
            request_builder.header(header::AUTHORIZATION, format!("Bearer {}", token));
    }

    match request_builder.json(&req).send() {
        Ok(res) => {
            let response_status = res.status();
            let response_text = res.text().expect("Failed to read response body");

            if response_status.is_success() {
                let result = serde_json::from_str::<SlackResponse>(&response_text)
                    .expect("Failed to parse response body");
                if result.ok {
                    if args.verbose {
                        println!("OK\n\n{}", &response_text);
                    }
                } else {
                    eprintln!(
                        "ERROR: {}\n\n{}",
                        result.error.unwrap_or("(null)".to_string()),
                        &response_text
                    );
                    std::process::exit(1);
                }
            } else {
                eprintln!("ERROR: {}\n{}", response_status, &response_text);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("ERROR\n\n{:?}", e);
            std::process::exit(1);
        }
    }
}
