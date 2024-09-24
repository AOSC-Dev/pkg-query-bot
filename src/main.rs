use std::fmt::Display;

use reqwest::Client;
use serde::Deserialize;
use teloxide::{prelude::*, utils::command::BotCommands};

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "Bot supports the following commands:"
)]
enum Cmd {
    #[command(description = "query a package (e,g: /query oma)")]
    Query(String),
}

#[derive(Debug, Deserialize)]
struct Pkg {
    name: String,
    version: String,
    description: String,
    versions: Vec<Version>,
}

impl Display for Pkg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "<b>{}</b>", self.name)?;
        writeln!(f, "<b>description</b>: {}", self.description)?;
        writeln!(f, "<b>Versions</b>:")?;
        for ver in &self.versions {
            write!(f, "  {}", ver)?;
            if ver.to_string() == self.version {
                writeln!(f, "(newest)")?;
            } else {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct Version {
    version: String,
    url: String,
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<a href=\"{}\">{}</a>", self.url, self.version)?;

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let bot = Bot::from_env();
    Cmd::repl(bot, answer).await;
}

async fn answer(bot: Bot, msg: Message, cmd: Cmd) -> ResponseResult<()> {
    match cmd {
        Cmd::Query(arg) => {
            let pkg = match get_pkg(&arg).await {
                Ok(pkg) => pkg,
                Err(e) => {
                    bot.send_message(msg.chat.id, e.to_string()).await?;
                    return Ok(());
                }
            };

            bot.send_message(msg.chat.id, pkg.to_string()).await?;
        }
    }

    Ok(())
}

async fn get_pkg(name: &str) -> anyhow::Result<Pkg> {
    let client = Client::builder().user_agent("bot").build()?;
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
