use std::{
    env,
    sync::{Arc, OnceLock},
};

use pkgsite_lib::PackagesSiteClient;
use runbot::prelude::{BotContextBuilder, loop_client};
use tracing::error;

use crate::qq::COMMAND;

mod package_site;
mod qq;

pub static PSC: OnceLock<Arc<PackagesSiteClient>> = OnceLock::new();

#[inline]
pub fn not_found_pkg(pkg: &str) -> String {
    format!(
        "Package {} not found\n\nDidn't find what you need? https://github.com/AOSC-Dev/aosc-os-abbs/issues/new?title=pakreq%3A%20{}&body=URL%3A%20%0A%0ADescription%3A%20",
        pkg, pkg
    )
}

#[inline]
pub fn not_match_pkg(pkg: &str) -> String {
    format!(
        "No matching package for {}\n\nDidn't find what you need? https://github.com/AOSC-Dev/aosc-os-abbs/issues/new?title=pakreq%3A%20{}&body=URL%3A%20%0A%0ADescription%3A%20",
        pkg, pkg
    )
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let client = Arc::new(PackagesSiteClient::from_env());
    let server = env::var("QQ_WS_SERVER").expect("QQ_WS_SERVER is not set");

    PSC.get_or_init(|| client.clone());
    qq(server).await;
}

async fn qq(server: String) {
    let bot = BotContextBuilder::new()
        // 声明链接地址
        .url(server)
        // 注册事件处理器 (方法名的UPPER_SNAKE)
        .add_processor(COMMAND)
        .build()
        .unwrap();

    if let Err(e) = loop_client(bot).await {
        error!("{e}");
    }
}
