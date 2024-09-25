use std::sync::Arc;

use package_site::PackageSiteClient;
use reqwest::StatusCode;
use teloxide::{prelude::*, types::ParseMode, utils::command::BotCommands};

mod package_site;

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "Bot supports the following commands:"
)]
enum Cmd {
    Help,
    #[command(description = "Get a package infomation (e,g: /pkg oma)")]
    Pkg(String),
    #[command(description = "Search packages (e,g: /search oma)")]
    Search(String),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();
    let bot = Bot::from_env();

    let client = Arc::new(PackageSiteClient::new(
        "https://packages.aosc.io".to_string(),
    )?);

    let handler =
        Update::filter_message().branch(dptree::entry().filter_command::<Cmd>().endpoint(
            |bot: Bot, msg: Message, cmd: Cmd, client: Arc<PackageSiteClient>| async move {
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

async fn answer(
    bot: Bot,
    msg: Message,
    cmd: Cmd,
    client: Arc<PackageSiteClient>,
) -> ResponseResult<()> {
    match cmd {
        Cmd::Pkg(arg) => {
            if arg.trim().is_empty() {
                return Ok(());
            }

            let pkg = match client.get_package(&arg).await {
                Ok(pkg) => pkg,
                Err(e) => {
                    if e.status().is_some_and(|x| x == StatusCode::NOT_FOUND) {
                        bot.send_message(msg.chat.id, format!("Package <b>{}</b> not found\n\nDidn't find what you need? <a href=\"https://github.com/AOSC-Dev/aosc-os-abbs/issues/new?title=pakreq%3A%20{}&body=URL%3A%20%0A%0ADescription%3A%20\">Request for the package</a>", arg, arg))
                            .parse_mode(ParseMode::Html)
                            .disable_web_page_preview(true)
                            .await?;
                        return Ok(());
                    }

                    bot.send_message(msg.chat.id, e.without_url().to_string())
                        .await?;
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
        Cmd::Search(arg) => {
            if arg.trim().is_empty() {
                return Ok(());
            }

            let result = match client.search(&arg).await {
                Ok(res) => res,
                Err(e) => {
                    bot.send_message(msg.chat.id, e.without_url().to_string())
                        .await?;
                    return Ok(());
                }
            };

            if result.is_empty() {
                bot.send_message(msg.chat.id, format!("No matching package for <b>{}</b>\n\nDidn't find what you need? <a href=\"https://github.com/AOSC-Dev/aosc-os-abbs/issues/new?title=pakreq%3A%20{}&body=URL%3A%20%0A%0ADescription%3A%20\">Request for the package</a>", arg, arg))
                    .parse_mode(ParseMode::Html)
                    .await?;
                return Ok(());
            }

            bot.send_message(msg.chat.id, result.fmt_result(&arg))
                .parse_mode(ParseMode::Html)
                .await?;
        }
    }

    Ok(())
}
