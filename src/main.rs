use std::collections::HashSet;
use std::env;
use dotenv::dotenv;

use serenity::async_trait;
use serenity::prelude::*;
use serenity::model::channel::Message;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{StandardFramework, CommandResult};
use serenity::model::id::UserId;
use serenity::model::prelude::{Activity, Embed};

#[group]
#[commands(ping)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: serenity::model::gateway::Ready) {
        println!("{} connected!", ready.user.tag());

        ctx.set_activity(Activity::playing("DM frog!help")).await;
    }
}

#[tokio::main]
async fn main() {
    dotenv().expect("Failed to load .env file");

    // Configure discord bot
    let d_framework = StandardFramework::new()
        .configure(|c| {
            let mut owner_ids = HashSet::new();
            let owner_id = env::var("DISCORD_OWNER_ID").unwrap_or_default();
            if let Some(owner_id_num) = owner_id.parse::<u64>().ok() {
                owner_ids.insert(UserId::from(owner_id_num));
            }

            c.prefix("frog!").owners(owner_ids)
        })
        .group(&GENERAL_GROUP);

    // Login discord bot
    let d_token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::non_privileged();
    let mut d_client = Client::builder(d_token, intents)
        .event_handler(Handler)
        .framework(d_framework)
        .await
        .expect("Error creating client");

    // Start a single discord bot shard
    if let Err(why) = d_client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}


