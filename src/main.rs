mod momo;
mod handler;
mod defs;

use std::env;
use std::sync::atomic::Ordering;
use teloxide::Bot;

rust_i18n::i18n!("locales", fallback = "en");
#[tokio::main]
async fn main() {
    println!("Available locales: {:?}", rust_i18n::available_locales!());
    if let Ok(use_ocr) = env::var("USE_OCR") {
        if use_ocr == "1" || use_ocr.to_lowercase() == "true" {
            println!("USE_OCR is enabled.");
            defs::USE_OCR.store(true, Ordering::Release)
        }
    }
    let bot = Bot::from_env();
    teloxide::repl(bot, handler::message_handler).await;
}