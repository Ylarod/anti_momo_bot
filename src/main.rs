mod momo;
mod handler;
mod defs;

use teloxide::Bot;

rust_i18n::i18n!("locales", fallback = "en");
#[tokio::main]
async fn main() {
    println!("Available locales: {:?}", rust_i18n::available_locales!());
    let bot = Bot::from_env();
    teloxide::repl(bot, handler::message_handler).await;
}