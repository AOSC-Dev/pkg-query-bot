use std::sync::Arc;

use pkgsite_lib::{PackagesSiteClient};
use teloxide::{
    Bot,
    payloads::SendMessageSetters,
    prelude::{Requester, ResponseResult},
    sugar::request::RequestLinkPreviewExt,
    types::{Message, ParseMode},
    utils::command::BotCommands,
};

use crate::{
    Cmd, not_found_pkg, not_match_pkg,
    package_site::{Pkg, SearchResult},
};

pub async fn answer(
    bot: Bot,
    msg: Message,
    cmd: Cmd,
    client: Arc<PackagesSiteClient>,
) -> ResponseResult<()> {
    match cmd {
        Cmd::Pkg(arg) => {
            if arg.trim().is_empty() {
                return Ok(());
            }

            let info = client.info(&[&arg]).await;
            let pkg = match info.as_deref() {
                Ok([pkg, ..]) => Pkg::from(pkg),
                Ok([]) => {
                    bot.send_message(msg.chat.id, not_found_pkg(&arg))
                        .parse_mode(ParseMode::Html)
                        .disable_link_preview(true)
                        .await?;
                    return Ok(());
                }
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
        Cmd::Search(arg) => {
            if arg.trim().is_empty() {
                return Ok(());
            }

            let search = client.search(&arg, true).await;
            let result = match search {
                Ok(Ok(res)) => SearchResult::from(res),
                Err(e) => {
                    bot.send_message(msg.chat.id, e.to_string()).await?;
                    return Ok(());
                }
                _ => unreachable!(), // redirect is off
            };

            if result.is_empty() {
                bot.send_message(msg.chat.id, not_match_pkg(&arg))
                    .parse_mode(ParseMode::Html)
                    .disable_link_preview(true)
                    .await?;
                return Ok(());
            }

            bot.send_message(msg.chat.id, result.fmt_result(&arg, &client.url))
                .parse_mode(ParseMode::Html)
                .disable_link_preview(true)
                .await?;
        }
    }

    Ok(())
}
