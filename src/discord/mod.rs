
use std::collections::HashSet;
use std::env;
use std::sync::Arc;

use serenity::{async_trait, CacheAndHttp};
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::model::id::UserId;
use serenity::framework::standard::{StandardFramework};
use serenity::http::CacheHttp;
use crate::ChannelJoinPartEvent;

use crate::twitch::TwitchMessageSimple;


mod com;
mod styled_str;


#[derive(Debug)]
pub struct TriggerEvent {
    pub receiver: u64,
    pub message: TwitchMessageSimple,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl TriggerEvent {
    pub fn new(receiver: u64, message: TwitchMessageSimple, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        Self {
            receiver,
            message,
            timestamp,
        }
    }
}

struct CommandPrefix;
impl TypeMapKey for CommandPrefix {
    type Value = String;
}

struct DbConnection;
impl TypeMapKey for DbConnection {
    type Value = Mutex<sqlx::pool::PoolConnection<sqlx::Sqlite>>;
}

struct IrcEventSender;
impl TypeMapKey for IrcEventSender {
    type Value = tokio::sync::mpsc::Sender<ChannelJoinPartEvent>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("`{}` connected!", ready.user.tag());
        
        let prefix = {
            let data = ctx.data.read().await;
            data.get::<CommandPrefix>().unwrap().clone()
        };

        ctx.set_activity(Activity::playing(format!("DM {}help", prefix))).await;
    }
}

pub async fn make_client(db_con: sqlx::pool::PoolConnection<sqlx::Sqlite>, irc_tx: tokio::sync::mpsc::Sender<ChannelJoinPartEvent>) -> Client {
    let prefix = env::var("DISCORD_PREFIX").unwrap_or_else(|_| "frog!".to_string());

    // Configure discord bot
    let d_framework = StandardFramework::new()
        .configure(|c| {
            let mut owner_ids = HashSet::new();
            let owner_id = env::var("DISCORD_OWNER_ID").unwrap_or_default();
            if let Ok(owner_id_num) = owner_id.parse::<u64>() {
                owner_ids.insert(UserId::from(owner_id_num));
            }

            c.prefix(prefix.clone()).owners(owner_ids)
        })
        .group(&com::GENERAL_GROUP)
        .group(&com::CHANNEL_GROUP)
        .group(&com::TRIGGER_GROUP);

    // Login discord bot
    let d_token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::non_privileged();
    let d_client = Client::builder(d_token, intents)
        .event_handler(Handler)
        .framework(d_framework)
        .await
        .expect("Error creating client");

    d_client.data.write().await.insert::<CommandPrefix>(prefix.clone());
    d_client.data.write().await.insert::<DbConnection>(Mutex::new(db_con));
    d_client.data.write().await.insert::<IrcEventSender>(irc_tx);

    d_client
}



// Notify user of the trigger event
pub async fn notify_user(cache_and_http: Arc<CacheAndHttp>, event: TriggerEvent) -> std::result::Result<(), serenity::Error> {
    UserId::from(event.receiver)
        .create_dm_channel(cache_and_http.clone()).await?
        .send_message(cache_and_http.http(),|m|
            m.embed(|e|
                e.description(event.message.message_highlighted("**"))
                    .author(|a|
                        a.name(format!("{} ∙ #{}", event.message.author, event.message.channel))
                            .url(format!("https://twitch.tv/{}", event.message.channel))
                    )
                    // .footer(|f|
                    //     f.text(format!("#{}", event.message.channel))
                    // )
                    .timestamp(event.timestamp)
            )
        ).await?;
    Ok(())
}
