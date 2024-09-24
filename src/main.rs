use std::fmt::Display;

use reqwest::Client;
use serde::Deserialize;
use teloxide::{prelude::*, types::ParseMode, utils::command::BotCommands};

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "Bot supports the following commands:"
)]
enum Cmd {
    Help,
    #[command(description = "Get a package infomation (e,g: /pkg oma)")]
    Pkg(String),
}

#[derive(Debug, Deserialize)]
struct Pkg {
    name: String,
    description: String,
    version_matrix: Vec<Version>,
}

impl Display for Pkg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "<b>{}</b>", self.name)?;
        writeln!(f)?;
        writeln!(f, "{}", self.description)?;
        writeln!(f)?;

        for v in &self.version_matrix {
            for m in &v.meta {
                if !m.hasmeta || m.version.is_empty() {
                    continue;
                }

                writeln!(f, "<b>{}</b>: <code>{}</code>", v.repo, m.version)?;

                break;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct Version {
    repo: String,
    meta: Vec<Meta>,
}

#[derive(Debug, Deserialize)]
struct Meta {
    hasmeta: bool,
    version: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let bot = Bot::from_env();
    let client = Client::builder().user_agent("bot").build()?;

    let handler =
        Update::filter_message().branch(dptree::entry().filter_command::<Cmd>().endpoint(
            |bot: Bot, msg: Message, cmd: Cmd, client: Client| async move {
                answer(bot, msg, cmd, client).await
            },
        ));

    Dispatcher::builder(bot.clone(), handler)
        // Pass the shared state to the handler as a dependency.
        .dependencies(dptree::deps![client])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

async fn answer(bot: Bot, msg: Message, cmd: Cmd, client: Client) -> ResponseResult<()> {
    match cmd {
        Cmd::Pkg(arg) => {
            let pkg = match get_pkg(&client, &arg).await {
                Ok(pkg) => pkg,
                Err(e) => {
                    bot.send_message(msg.chat.id, e.to_string()).await?;
                    return Ok(());
                }
            };

            bot.send_message(msg.chat.id, pkg.to_string())
                .parse_mode(ParseMode::Html)
                .await?;
        }
        Cmd::Help => {
            bot.send_message(msg.chat.id, Cmd::descriptions().to_string())
                .await?;
        }
    }

    Ok(())
}

async fn get_pkg(client: &Client, name: &str) -> anyhow::Result<Pkg> {
    Ok(client
        .get(format!(
            "https://packages.aosc.io/packages/{}?type=json",
            name
        ))
        .send()
        .await?
        .error_for_status()?
        .json::<Pkg>()
        .await?)
}
