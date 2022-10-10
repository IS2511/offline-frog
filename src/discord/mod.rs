
use std::collections::HashSet;
use std::env;
use std::sync::Arc;

use serenity::{async_trait, CacheAndHttp};
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::model::id::UserId;
use serenity::framework::standard::{StandardFramework};
use serenity::http::CacheHttp;
use crate::discord::com::{get_bot_prefix, update_channel_count};
use crate::IrcMessageEvent;

use crate::twitch::TwitchMessageSimple;


mod com;
mod styled_str;
mod extra;


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

macro_rules! make_type_key {
    ($name:ident, $t:ty) => {
        struct $name;
        impl TypeMapKey for $name {
            type Value = $t;
        }
    };
}

make_type_key!(ChannelCount, i32);
make_type_key!(CommandPrefix, String);
make_type_key!(DbConnection, Mutex<sqlx::pool::PoolConnection<sqlx::Sqlite>>);
make_type_key!(IrcEventSender, tokio::sync::mpsc::Sender<IrcMessageEvent>);

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("`{}` connected!", ready.user.tag());

        update_channel_count!(ctx, 0);
    }
}

pub async fn make_client(mut db_con: sqlx::pool::PoolConnection<sqlx::Sqlite>, irc_tx: tokio::sync::mpsc::Sender<IrcMessageEvent>) -> Client {
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
        .group(&com::TRIGGER_GROUP)
        .group(&com::IGNORE_GROUP);

    // Login discord bot
    let d_token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::non_privileged();
    let d_client = Client::builder(d_token, intents)
        .event_handler(Handler)
        .framework(d_framework)
        .await
        .expect("Error creating client");


    let channel_count = sqlx::query!("SELECT COUNT(DISTINCT channel) as count FROM channels")
        .fetch_one(&mut db_con)
        .await
        .expect("Error counting channels in DB")
        .count;

    {
        let mut data = d_client.data.write().await;
        data.insert::<ChannelCount>(channel_count);
        data.insert::<CommandPrefix>(prefix.clone());
        data.insert::<DbConnection>(Mutex::new(db_con));
        data.insert::<IrcEventSender>(irc_tx);
    }

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
