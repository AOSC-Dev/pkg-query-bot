use std::{
    sync::{Arc, OnceLock},
};

use pkgsite_lib::PackagesSiteClient;
use teloxide::{
    Bot,
    dispatching::{HandlerExt, UpdateFilterExt},
    dptree,
    prelude::Dispatcher,
    types::{Message, Update},
    utils::command::BotCommands,
};

mod tg;
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

pub static PSC: OnceLock<Arc<PackagesSiteClient>> = OnceLock::new();

#[inline]
pub fn not_found_pkg(pkg: &str) -> String {
    format!(
        "Package <b>{}</b> not found\n\nDidn't find what you need? <a href=\"https://github.com/AOSC-Dev/aosc-os-abbs/issues/new?title=pakreq%3A%20{}&body=URL%3A%20%0A%0ADescription%3A%20\">Request for the package</a>",
        pkg, pkg
    )
}

#[inline]
pub fn not_match_pkg(pkg: &str) -> String {
    format!(
        "No matching package for <b>{}</b>\n\nDidn't find what you need? <a href=\"https://github.com/AOSC-Dev/aosc-os-abbs/issues/new?title=pakreq%3A%20{}&body=URL%3A%20%0A%0ADescription%3A%20\">Request for the package</a>",
        pkg, pkg
    )
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let client = Arc::new(PackagesSiteClient::from_env().unwrap());

    tg(client).await;
}

async fn tg(client: Arc<PackagesSiteClient>) {
    let bot = Bot::from_env();

    let handler =
        Update::filter_message().branch(dptree::entry().filter_command::<Cmd>().endpoint(
            |bot: Bot, msg: Message, cmd: Cmd, client: Arc<PackagesSiteClient>| async move {
                tg::answer(bot, msg, cmd, client).await
            },
        ));

    Dispatcher::builder(bot.clone(), handler)
        // Pass the shared state to the handler as a dependency.
        .dependencies(dptree::deps![client])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
