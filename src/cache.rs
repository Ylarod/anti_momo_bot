use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use teloxide::prelude::Requester;
use teloxide::types::{ChatId, ChatMember, UserId};
use teloxide::Bot;
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub struct PermCache {
    bot: Bot,
    bot_id: UserId,
    bot_admin: Arc<RwLock<HashMap<ChatId, bool>>>,
    user_admin: Arc<RwLock<HashMap<ChatId, HashMap<UserId, bool>>>>,
}

impl PermCache {
    pub async fn new(bot: &Bot) -> Self {
        PermCache {
            bot: bot.clone(),
            bot_id: bot.get_me().await.unwrap().id,
            bot_admin: Arc::new(RwLock::new(HashMap::new())),
            user_admin: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn is_bot_admin(&self, chat_id: ChatId) -> Result<bool> {
        let ptr = self.bot_admin.clone();
        let lock = ptr.read().await;
        let result = lock.get(&chat_id);
        match result {
            None => {
                drop(lock);
                Ok(self.update_bot_admin(chat_id).await?)
            },
            Some(perm) => Ok(*perm),
        }
    }

    pub async fn set_bot_admin(&self, chat_id: ChatId, is_admin: bool) -> Result<()> {
        self.bot_admin
            .clone()
            .write()
            .await
            .insert(chat_id, is_admin);
        Ok(())
    }

    async fn update_bot_admin(&self, chat_id: ChatId) -> Result<bool> {
        let perm = self
            .bot
            .get_chat_member(chat_id, self.bot_id)
            .await
            .map(|member: ChatMember| member.can_restrict_members())?;
        self.bot_admin.clone().write().await.insert(chat_id, perm);
        Ok(perm)
    }

    pub async fn is_user_admin(&self, user_id: UserId, chat_id: ChatId) -> Result<bool> {
        let ptr = self.user_admin.clone();
        let lock = ptr.read().await;
        let result = lock.get(&chat_id);
        return match result {
            None => {
                drop(lock);
                Ok(self.update_user_admin(user_id, chat_id).await?)
            },
            Some(perm) => match perm.get(&user_id) {
                None => {
                    drop(lock);
                    Ok(self.update_user_admin(user_id, chat_id).await?)
                },
                Some(perm) => Ok(*perm),
            },
        };
    }

    async fn update_user_admin(&self, user_id: UserId, chat_id: ChatId) -> Result<bool> {
        let perm = self
            .bot
            .get_chat_member(chat_id, user_id)
            .await
            .map(|member: ChatMember| member.is_owner() || member.is_administrator())?;
        log::debug!("user_id: {}, chat_id: {}, is_admin: {}", user_id, chat_id, perm);
        self.user_admin
            .clone()
            .write()
            .await
            .entry(chat_id)
            .or_insert(HashMap::new())
            .insert(user_id, perm);
        Ok(perm)
    }
}
