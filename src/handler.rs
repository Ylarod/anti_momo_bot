use std::ops::Add;
use rust_i18n::t;
use teloxide::Bot;
use teloxide::prelude::{Message, Requester, ResponseResult};
use teloxide::net::Download;
use teloxide::types::{ChatMember, ChatPermissions};
use teloxide::payloads::{RestrictChatMemberSetters, SendMessageSetters};
use crate::{momo, defs};


pub async fn message_handler(bot: Bot, message: Message) -> ResponseResult<()>{
    if let Some(user) = message.from(){
        if let Some(lang) = &user.language_code {
            // println!("User lang: {lang}");
            rust_i18n::set_locale(lang);
        }
    }
    if let Some(photos) = message.photo() {
        if let Some(user) = message.from(){
            let is_admin = bot
                .get_chat_member(message.chat.id, user.id)
                .await
                .map(|member: ChatMember| member.is_administrator())?;
            if is_admin {
                return Ok(());
            }
        }
        if photos.len() > 1 {
            let photo = &photos[0];
            let path = bot.get_file(photo.file.id.clone()).await?.path;
            let local_path = format!("/tmp/{}", path);
            let mut dst = tokio::fs::File::create(&local_path).await?;
            bot.download_file(path.as_str(), &mut dst).await?;
            if momo::is_momo_screenshot(&local_path.as_str()).ok() == Some(true){
                let bot_id = bot.get_me().await?.id;
                let can_mute = bot
                    .get_chat_member(message.chat.id, bot_id)
                    .await
                    .map(|member: ChatMember| member.can_restrict_members())?;
                if can_mute {
                    if let Some(user) = message.from(){
                        bot.restrict_chat_member(message.chat.id, user.id, ChatPermissions::empty())
                            .until_date(chrono::Utc::now().add(chrono::Duration::seconds(defs::BAN_DURATION))).await?;
                        bot.send_message(message.chat.id, t!("restrict"))
                            .reply_to_message_id(message.id).await?;
                    }else{
                        bot.send_message(message.chat.id, t!("restrict_failed"))
                            .reply_to_message_id(message.id).await?;
                    }
                } else {
                    bot.send_message(message.chat.id, t!("momo_found"))
                        .reply_to_message_id(message.id).await?;
                }
            }
        }
    }
    Ok(())
}