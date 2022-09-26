use dotenv::dotenv;
use sqlx::sqlite::SqlitePoolOptions;

mod discord;
mod twitch;
// mod db;

// use db::KvStore;

#[tokio::main]
async fn main() {
    dotenv().expect("Failed to load .env file");

    // let db = db::connect().await.expect("Failed to connect to database");
    //
    // let discord_db = Arc::new(db);
    // let twitch_db = discord_db.clone();

    // db.set("test".to_string(), "test".to_string()).await;

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:local.db?mode=rwc")
        .await
        .expect("Failed to connect to database");

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS channels
                (
                    id              INTEGER PRIMARY KEY,
                    discord_user_id STRING NOT NULL,
                    channel         STRING NOT NULL
                )
            "#).execute(&pool).await.expect("create channels table");

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS triggers
                (
                    id              INTEGER PRIMARY KEY,
                    discord_user_id STRING NOT NULL,
                    trigger         STRING NOT NULL,
                    case_sensitive  BOOLEAN DEFAULT FALSE,
                    regex           BOOLEAN DEFAULT FALSE
                )
            "#).execute(&pool).await.expect("create triggers table");

    // Run discord bot
    let discord_handle = tokio::spawn(async move {
        discord::start().await
    });

    // Run twitch listener
    let twitch_handle = tokio::spawn(async move {
        twitch::start().await
    });

    discord_handle.await.expect("Discord thread panicked");
    twitch_handle.await.expect("Twitch thread panicked");

}

