use sqlx::prelude::*;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Sqlite};


pub async fn setup() -> Result<Pool<Sqlite>, sqlx::Error> {

    let pool = SqlitePoolOptions::new()
        .min_connections(2)
        .max_connections(3)
        .connect("sqlite:local.db?mode=rwc")
        .await
        .expect("Failed to connect to database");

    let tx = pool.begin().await?;

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS channels
                (
                    id              INTEGER PRIMARY KEY,
                    discord_user_id STRING NOT NULL,
                    channel         STRING NOT NULL
                )
            "#).execute(&pool).await?;

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS triggers
                (
                    id              INTEGER PRIMARY KEY,
                    discord_user_id STRING NOT NULL,
                    trigger         STRING NOT NULL,
                    case_sensitive  BOOLEAN DEFAULT FALSE,
                    regex           BOOLEAN DEFAULT FALSE
                )
            "#).execute(&pool).await?;

    tx.commit().await?;

    Ok(pool)
}
