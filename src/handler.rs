use crate::cache::PermCache;
use crate::{defs, momo};
use anyhow::{Context, Result};
use reqwest::Url;
use rust_i18n::t;
use std::ops::Add;
use teloxide::net::Download;
use teloxide::payloads::{RestrictChatMemberSetters, SendMessageSetters};
use teloxide::prelude::{Message, Requester};
use teloxide::types::{
    ChatMemberUpdated, ChatPermissions, InlineKeyboardButton, InlineKeyboardMarkup, ParseMode, User,
};
use teloxide::utils::command::BotCommands;
use teloxide::Bot;

pub struct TranslateContext {
    locale: String,
}

impl TranslateContext {
    fn new() -> Self {
        TranslateContext {
            locale: "en".to_string(),
        }
    }

    fn from_message(message: &Message) -> Self {
        if let Some(user) = &message.from() {
            TranslateContext::from_user(user)
        } else {
            TranslateContext::new()
        }
    }

    fn from_user(user: &User) -> Self {
        let lang = if let Some(lang) = &user.language_code {
            log::trace!("User lang: {lang}");
            lang.as_str()
        } else {
            "en"
        };
        TranslateContext {
            locale: lang.to_string(),
        }
    }

    fn t(&self, key: &str) -> String {
        t!(key, locale = self.locale.as_str())
    }
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    Start,

    #[command(description = "display this text")]
    Help,
}

pub async fn permission_handler(
    bot: Bot,
    update: ChatMemberUpdated,
    perm: PermCache,
) -> Result<()> {
    if update.new_chat_member.can_restrict_members()
        != update.old_chat_member.can_restrict_members()
    {
        let ctx = TranslateContext::from_user(&update.from);
        perm.set_bot_admin(
            update.chat.id,
            update.new_chat_member.can_restrict_members(),
        )
        .await?;
        bot.send_message(update.chat.id, ctx.t("bot_perm_update"))
            .await?;
    }
    Ok(())
}

pub async fn command_handler(bot: Bot, message: Message, command: Command) -> Result<()> {
    let ctx = TranslateContext::from_message(&message);
    match command {
        Command::Start => {
            let msg = format!("{}", ctx.t("welcome_body"),);
            let username = bot
                .get_me()
                .await?
                .username
                .clone()
                .context("antimomobot")?;
            let url = format!(
                "https://t.me/{}?startgroup=start&admin=can_restrict_members",
                username.as_str()
            );
            let btn =
                InlineKeyboardButton::url(ctx.t("welcome_setmeasadmin"), Url::parse(url.as_str())?);
            let btns = [btn];
            bot.send_message(message.chat.id, msg)
                .reply_to_message_id(message.id)
                .disable_web_page_preview(true)
                .parse_mode(ParseMode::Html)
                .reply_markup(InlineKeyboardMarkup::new([btns]))
                .await?;
        }
        Command::Help => {
            bot.send_message(message.chat.id, Command::descriptions().to_string())
                .await?;
        }
    };
    Ok(())
}

pub async fn message_handler(bot: Bot, message: Message, perm: PermCache) -> Result<()> {
    log::debug!(
        "[{}] {}: {}",
        message.chat.title().unwrap_or(""),
        message.from().context("")?.first_name,
        message.text().unwrap_or("")
    );
    let ctx = TranslateContext::from_message(&message);
    if let Some(photos) = message.photo() {
        log::trace!("message contains photo");
        if let Some(user) = message.from() {
            log::trace!("check user admin");
            if perm.is_user_admin(user.id, message.chat.id).await? {
                log::trace!("user is admin, skip");
                return Ok(());
            }
        }
        log::trace!("processing photo");
        if photos.len() > 1 {
            let photo = &photos[0];
            log::trace!("get file path");
            let path = bot.get_file(photo.file.id.clone()).await?.path;
            let local_path = format!("/tmp/{}", path);
            let mut dst = tokio::fs::File::create(&local_path).await?;
            log::trace!("downloading picture");
            bot.download_file(path.as_str(), &mut dst).await?;
            log::trace!("detecting momo");
            if momo::is_momo_screenshot(local_path.as_str()).ok() == Some(true) {
                log::trace!("momo found");
                if perm.is_bot_admin(message.chat.id).await? {
                    log::trace!("bot is admin, mute");
                    if let Some(user) = message.from() {
                        bot.restrict_chat_member(
                            message.chat.id,
                            user.id,
                            ChatPermissions::empty(),
                        )
                        .until_date(
                            chrono::Utc::now().add(chrono::Duration::seconds(defs::BAN_DURATION)),
                        )
                        .await?;
                        bot.send_message(message.chat.id, ctx.t("restrict"))
                            .reply_to_message_id(message.id)
                            .await?;
                    } else {
                        bot.send_message(message.chat.id, ctx.t("restrict_failed"))
                            .reply_to_message_id(message.id)
                            .await?;
                    }
                } else {
                    log::trace!("bot is not admin, reply");
                    bot.send_message(message.chat.id, ctx.t("momo_found"))
                        .reply_to_message_id(message.id)
                        .await?;
                }
            }
        }
    }
    Ok(())
}
