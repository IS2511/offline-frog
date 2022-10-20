use std::sync::Arc;
use dotenvy::dotenv;
use tokio::sync::{mpsc, RwLock};

mod discord;
mod twitch;
mod db;

use discord::TriggerEvent;
use crate::twitch::IrcMessageEvent;

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
    let (irc_tx, mut irc_rx) = mpsc::channel::<IrcMessageEvent>(10_000);
    let irc_tx_for_irc = irc_tx.clone();

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

    let twitch_client = twitch::make_client(twitch_db_con, discord_tx).await.expect("Failed to make twitch client");
    let twitch_client = Arc::new(RwLock::new(twitch_client));
    let twitch_client_clone = twitch_client.clone();
    let mut twitch_msg_stream = twitch_client.write().await.stream().expect("Failed to get twitch message stream");


    // Run twitch listener
    let twitch_handle = tokio::spawn(async move {
        use futures_util::StreamExt; // for next()
        while let Some(message) = twitch_msg_stream.next().await {
            if let Ok(message) = message {
                let res = irc_tx_for_irc.send(IrcMessageEvent::Incoming(message)).await;
                if res.is_err() {
                    println!("[IRC] Error sending message to irc thread: {:?}", res.err().unwrap());
                }
            } else {
                let err = message.err().unwrap();
                println!("[IRC] Error getting message from irc stream: {:?}", &err);
                match err {
                    irc::error::Error::PingTimeout |
                    irc::error::Error::AsyncChannelClosed |
                    irc::error::Error::Tls(_) => {
                        println!("[IRC] Connection error, reconnecting...");
                        twitch_client_clone.write().await
                            .restart().await.expect("Failed to restart twitch client");
                    },
                    _ => {}
                }
            }
        }
    });

    tokio::spawn(async move {
        while let Some(event) = irc_rx.recv().await {
            match event {
                IrcMessageEvent::Incoming(message) => {
                    let res = twitch_client.write().await.handle(&message).await;
                    if let Err(e) = res {
                        println!("[IRC] Error handling message: {:?}", e);
                    }
                }
                IrcMessageEvent::Outgoing(message) => {
                    // println!("Sending message: {:?}", message);
                    let res = twitch_client.read().await.send(message);
                    if let Err(e) = res {
                        println!("[IRC] Error sending message: {:?}", e);
                        println!("[IRC] Restarting client just in case");
                        twitch_client.write().await
                            .restart().await.expect("Failed to restart twitch client");
                    }
                }
            }
        }
    });

    discord_handle.await.expect("Discord thread panicked");
    twitch_handle.await.expect("Twitch thread panicked");

}

