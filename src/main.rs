use std::sync::Arc;
use dotenv::dotenv;
use tokio::sync::mpsc;

mod discord;
mod twitch;
mod db;

use discord::TriggerEvent;
use crate::twitch::ChannelJoinPartEvent;

#[tokio::main]
async fn main() {
    dotenv().expect("Failed to load .env file");

    let db_pool = db::setup()
        .await.expect("Failed to setup database");
    let discord_db_con = db_pool.acquire()
        .await.expect("Failed to acquire database connection");
    let twitch_db_con = db_pool.acquire()
        .await.expect("Failed to acquire database connection");

    let (discord_tx, mut discord_rx) = mpsc::channel::<TriggerEvent>(10_000);
    let (irc_tx, mut irc_rx) = mpsc::channel::<ChannelJoinPartEvent>(1_000);

    // Run discord bot
    let discord_handle = tokio::spawn(async move {
        let mut client = discord::make_client(discord_db_con, irc_tx).await;

        let cache_and_http = client.cache_and_http.clone();

        tokio::spawn(async move {
            while let Some(event) = discord_rx.recv().await {
                match discord::notify_user(cache_and_http.clone(), event).await {
                    Ok(_) => {},
                    Err(e) => {
                        println!("[DS] Error sending direct message: {}", e);
                    }
                }
            }
        });

        if let Err(why) = client.start().await {
            println!("[DS] An error occurred while running the client: {:?}", why);
        }
    });

    // Run twitch listener
    let twitch_handle = tokio::spawn(async move {
        let mut client = twitch::make_client(twitch_db_con, discord_tx).await.expect("Failed to make twitch client");
        let mut client = Arc::new(tokio::sync::RwLock::new(client));
        let client_clone = client.clone();

        let irc_sender_thread = tokio::spawn(async move {
            while let Some(event) = irc_rx.recv().await {
                match event {
                    // TODO: Handle join/part errors
                    ChannelJoinPartEvent::Join(channel) => {
                        println!("[IRC] Joining channel #{}...", channel);
                        client_clone.read().await.join_channel(&channel).await;
                    },
                    ChannelJoinPartEvent::Part(channel) => {
                        println!("[IRC] Parting channel #{}...", channel);
                        client_clone.read().await.part_channel(&channel).await;
                    },
                }
            }
        });

        if let Err(why) = client.write().await.start().await {
            println!("[IRC] An error occurred while running the client: {:?}", why);
        }
        irc_sender_thread.await.expect("Failed to join irc_sender_thread");
    });

    discord_handle.await.expect("Discord thread panicked");
    twitch_handle.await.expect("Twitch thread panicked");

}

