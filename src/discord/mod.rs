
use std::collections::HashSet;
use std::env;

use serenity::async_trait;
use serenity::prelude::*;
use serenity::framework::standard::{StandardFramework};
use serenity::model::id::UserId;
use serenity::model::prelude::{Activity};

mod com;

struct CommandPrefix;
impl TypeMapKey for CommandPrefix {
    type Value = String;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: serenity::model::gateway::Ready) {
        println!("`{}` connected!", ready.user.tag());

        ctx.set_activity(Activity::playing("DM frog!help")).await;
    }
}

pub async fn start() {
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

    // Start a single discord bot shard
    if let Err(why) = d_client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
