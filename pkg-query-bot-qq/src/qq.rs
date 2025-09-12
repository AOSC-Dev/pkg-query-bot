use std::sync::Arc;

use pkgsite_lib::SearchExactMatch;
use runbot::event::SendMessage;
use runbot::prelude::MessageProcessor;
use runbot::prelude::Processor;
use runbot::{
    event::{Message, MessageType},
    prelude::{BotContext, processor},
};

use crate::PACKAGE_SITE_CLIENT;
use crate::not_found_pkg;
use crate::not_match_pkg;
use crate::package_site::Pkg;
use crate::package_site::SearchResult;

#[processor]
pub async fn command(bot_ctx: Arc<BotContext>, message: &Message) -> anyhow::Result<bool> {
    let msg = message.raw_message.trim();

    if let Some(p) = msg.strip_prefix("/pkg ") {
        pkg(bot_ctx, message, p).await
    } else if let Some(keyword) = msg.strip_prefix("/search ") {
        search(bot_ctx, message, keyword).await
    } else {
        Ok(true)
    }
}

pub async fn pkg(bot_ctx: Arc<BotContext>, message: &Message, pkg: &str) -> anyhow::Result<bool> {
    let client = &*PACKAGE_SITE_CLIENT;
    let info = client.info(&[&pkg]).await;
    let pkg = match info.as_deref() {
        Ok([pkg, ..]) => Pkg::from(pkg),
        Ok([]) => {
            send_message(bot_ctx, message, not_found_pkg(&pkg)).await?;
            return Ok(true);
        }
        Err(e) => {
            send_message(bot_ctx, message, e.to_string()).await?;
            return Ok(false);
        }
    };

    send_message(bot_ctx, message, pkg.to_string()).await?;

    Ok(true)
}

pub async fn search(
    bot_ctx: Arc<BotContext>,
    message: &Message,
    keyword: &str,
) -> anyhow::Result<bool> {
    let client = &*PACKAGE_SITE_CLIENT;
    let search = client.search(keyword, true).await;
    let result = match search {
        Ok(SearchExactMatch::Search(ref res)) => SearchResult::from(res),
        Err(e) => {
            send_message(bot_ctx, message, e.to_string()).await?;
            return Ok(false);
        }
        _ => unreachable!(), // redirect is off
    };

    if result.is_empty() {
        send_message(bot_ctx, message, not_match_pkg(keyword)).await?;
        return Ok(false);
    }

    send_message(bot_ctx, message, result.fmt_result(keyword, &client.url)).await?;

    Ok(true)
}

async fn send_message(
    bot_ctx: Arc<BotContext>,
    message: &Message,
    send: impl SendMessage,
) -> anyhow::Result<()> {
    tracing::debug!("{message:#?}");
    match &message.message_type {
        MessageType::Private => {
            bot_ctx.send_private_message(message.user_id, send).await?;
        }
        MessageType::Group => {
            bot_ctx.send_group_message(message.group_id, send).await?;
        }
        MessageType::Unknown(u) => {
            tracing::info!("Got unknown message: {u}");
        }
    }

    Ok(())
}
