mod cache;
mod defs;
mod handler;
mod momo;

use std::env;
use std::sync::atomic::Ordering;
use teloxide::dispatching::{HandlerExt, UpdateFilterExt};
use teloxide::dptree::deps;
use teloxide::prelude::Dispatcher;
use teloxide::types::Update;
use teloxide::{dptree, Bot};

rust_i18n::i18n!("locales", fallback = "en");
#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Available locales: {:?}", rust_i18n::available_locales!());
    if let Ok(use_ocr) = env::var("USE_OCR") {
        if use_ocr == "1" || use_ocr.to_lowercase() == "true" {
            println!("USE_OCR is enabled.");
            defs::USE_OCR.store(true, Ordering::Release)
        }
    }

    let bot = Bot::from_env();

    let perm_cache = cache::PermCache::new(&bot).await;

    let message_handler = Update::filter_message().endpoint(handler::message_handler);

    let command_handler = Update::filter_message()
        .filter_command::<handler::Command>()
        .endpoint(handler::command_handler);

    let permission_handler = Update::filter_my_chat_member().endpoint(handler::permission_handler);

    let handler = dptree::entry()
        .branch(permission_handler)
        .branch(command_handler)
        .branch(message_handler);

    Dispatcher::builder(bot, handler)
        .dependencies(deps![perm_cache])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
