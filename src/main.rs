use dotenv::dotenv;
use tokio::sync::mpsc;

mod discord;
mod twitch;
mod db;

use discord::DiscordMessageRequest;

#[tokio::main]
async fn main() {
    dotenv().expect("Failed to load .env file");

    // let db = db::connect().await.expect("Failed to connect to database");
    //
    // let discord_db = Arc::new(db);
    // let twitch_db = discord_db.clone();

    // db.set("test".to_string(), "test".to_string()).await;

    let db_pool = db::setup()
        .await.expect("Failed to setup database");

    let discord_db_con = db_pool.acquire()
        .await.expect("Failed to acquire database connection");
    let twitch_db_con = db_pool.acquire()
        .await.expect("Failed to acquire database connection");

    let (discord_tx, mut discord_rx) = mpsc::channel::<Box<DiscordMessageRequest>>(10_000);

    // Run discord bot
    let discord_handle = tokio::spawn(async move {
        let client = discord::start(discord_db_con, discord_rx).await;

        let cache_and_http = client.cache_and_http.clone();

        tokio::spawn(async move {
            while let Some(mut msg) = discord_rx.recv().await {
                match discord::send_discord_dm(cache_and_http.clone(), msg).await {
                    Ok(_) => {},
                    Err(e) => {
                        println!("[DS] Error sending direct message: {}", e);
                    }
                }
            }
        });
    });

    // Run twitch listener
    let twitch_handle = tokio::spawn(async move {
        twitch::start(twitch_db_con, discord_tx).await
    });

    discord_handle.await.expect("Discord thread panicked");
    twitch_handle.await.expect("Twitch thread panicked");

}

