
use std::collections::HashSet;
use std::env;
use std::sync::Arc;

use serenity::{async_trait, CacheAndHttp};
use serenity::builder::CreateMessage;
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::model::id::UserId;
use serenity::framework::standard::{StandardFramework};
use serenity::http::CacheHttp;
use serenity::utils::MessageBuilder;

use tokio::sync::mpsc;

pub struct DiscordMessageRequest<'a> {
    pub user_id: UserId,
    pub message: CreateMessage<'a>,
}


mod com;

struct CommandPrefix;
impl TypeMapKey for CommandPrefix {
    type Value = String;
}

// struct KvStorageTMK;
// impl TypeMapKey for KvStorageTMK {
//     type Value = Arc<KvStorage>;
// }

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("`{}` connected!", ready.user.tag());

        ctx.set_activity(Activity::playing("DM frog!help")).await;
    }
}

pub async fn start(db_con: sqlx::pool::PoolConnection<sqlx::Sqlite>, mut rx: mpsc::Receiver<Box<DiscordMessageRequest<'_>>>) -> Client {
    let prefix = env::var("DISCORD_PREFIX").unwrap_or_else(|_| "frog!".to_string());

    // Configure discord bot
    let d_framework = StandardFramework::new()
        .configure(|c| {
            let mut owner_ids = HashSet::new();
            let owner_id = env::var("DISCORD_OWNER_ID").unwrap_or_default();
            if let Some(owner_id_num) = owner_id.parse::<u64>().ok() {
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
    let mut d_client = Client::builder(d_token, intents)
        .event_handler(Handler)
        .framework(d_framework)
        .await
        .expect("Error creating client");

    d_client.data.write().await.insert::<CommandPrefix>(prefix.clone());
    // d_client.data.write().await.insert::<KvStorageTMK>(db);

    let cache_and_http = d_client.cache_and_http.clone();
    // let rx = Arc::new(RwLock::new(rx));


    d_client

    // // Start a single discord bot shard
    // if let Err(why) = d_client.start().await {
    //     println!("[DS] An error occurred while running the client: {:?}", why);
    // }

    // sender_handle.await.expect("Discord sender thread panicked");
}

pub async fn send_discord_dm(cache_and_http: Arc<CacheAndHttp>, mut msg: Box<DiscordMessageRequest<'_>>) -> std::result::Result<(), serenity::Error> {
    msg.user_id
        .create_dm_channel(cache_and_http.clone()).await?
        .send_message(cache_and_http.http(),|m|
            &mut msg.message
        ).await?;
    Ok(())
}
