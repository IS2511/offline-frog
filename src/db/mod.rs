use std::env;
// use sqlx::prelude::*;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Sqlite};


pub struct TriggerRecordNoDiscord {
    pub id: i64,
    pub trigger: String,
    pub case_sensitive: bool,
    pub regex: bool,
}

pub struct TriggerRecordNoId {
    pub discord_user_id: i64,
    pub trigger: String,
    pub case_sensitive: bool,
    pub regex: bool,
}

pub async fn setup() -> Result<Pool<Sqlite>, sqlx::Error> {

    let pool = SqlitePoolOptions::new()
        .min_connections(2)
        .max_connections(3)
        .connect(&env::var("DATABASE_URL").expect("DATABASE_URL must be set"))
        .await
        .expect("Failed to connect to database");

    let tx = pool.begin().await?;

    sqlx::query!(
        r#"CREATE TABLE IF NOT EXISTS channels
                (
                    id              INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                    discord_user_id INTEGER NOT NULL,
                    channel         TEXT NOT NULL,
                    UNIQUE(discord_user_id, channel) ON CONFLICT FAIL
                )
            "#).execute(&pool).await?;

    sqlx::query!(
        r#"CREATE TABLE IF NOT EXISTS triggers
                (
                    id              INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                    discord_user_id INTEGER NOT NULL,
                    trigger         TEXT NOT NULL,
                    case_sensitive  BOOLEAN DEFAULT FALSE NOT NULL,
                    regex           BOOLEAN DEFAULT FALSE NOT NULL,
                    UNIQUE(discord_user_id, trigger, regex) ON CONFLICT FAIL
                )
            "#).execute(&pool).await?;

    tx.commit().await?;

    Ok(pool)
}
