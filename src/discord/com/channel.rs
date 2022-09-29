use std::borrow::Cow;
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::{command, group};

use clap::{Parser, Subcommand};
use sqlx::{Acquire, Row};

use crate::discord::{CommandPrefix, DbConnection, styled_str};
use crate::discord::com::{get_bot_prefix, get_db};

/// Arguments to the channel command
#[derive(clap::Parser, Debug)]
struct Args {
    /// Action to perform
    #[command(subcommand)]
    action: Actions,
}

#[derive(Subcommand, Debug)]
enum Actions {
    /// Add channels to the list of monitored channels
    Add {
        /// Channels to add
        channels: Vec<String>,
    },
    /// Remove channels from the list of monitored channels
    Remove {
        /// Channel to remove
        channels: Vec<String>,
    },
    /// List all channels that are monitored
    List,
}

#[group]
#[commands(channel)]
struct Channel;

#[command]
async fn channel(ctx: &Context, msg: &Message) -> CommandResult {
    let prefix = get_bot_prefix!(ctx);

    let args = Args::try_parse_from(msg.content.trim_start_matches(&prefix).split_whitespace());

    get_db!(ctx, db_con);

    match args {
        Ok(args) => {
            match args.action {
                Actions::Add { channels } => {
                    let mut tx = db_con.begin().await?;
                    for channel in &channels {
                        let res = sqlx::query("INSERT OR IGNORE INTO channels (discord_user_id, channel) VALUES (?, ?)")
                            .bind(msg.author.id.0 as i64)
                            .bind(channel)
                            .execute(&mut tx).await;
                        if let Err(e) = res {
                            match e {
                                sqlx::Error::Database(e) => {
                                    let code = e.code().unwrap_or(Cow::Borrowed(""));
                                    if code == "2067" { // SQLITE_CONSTRAINT_UNIQUE (UNIQUE constraint failed)
                                        msg.reply(ctx, format!("Channel {} is already in the list", channel)).await?;
                                    } else {
                                        // msg.reply(ctx, format!("Error adding channel #{}: {}", channel, e.message())).await?;
                                        msg.reply(ctx, format!("Error adding channel #{}", channel)).await?;
                                    }
                                },
                                _ => {
                                    // msg.reply(ctx, format!("Error adding channel {}: {:?}", &channel, e)).await?;
                                    msg.reply(ctx, format!("Error adding channel #{}", channel)).await?;
                                }
                            }
                        }
                    }
                     match tx.commit().await {
                        Ok(_) => { msg.reply(ctx, "Added channels").await?; },
                         // TODO: Make so no data leaks through the error message
                        Err(e) => { msg.reply(ctx, format!("Error adding channels: {:?}", e)).await?; },
                     }
                },
                Actions::Remove { channels } => {
                    let mut tx = db_con.begin().await?;
                    for channel in &channels {
                        let res = sqlx::query("DELETE FROM channels WHERE discord_user_id = ? AND channel = ?")
                            .bind(msg.author.id.0 as i64)
                            .bind(channel)
                            .execute(&mut tx).await;
                        if let Err(e) = res {
                            // msg.reply(ctx, format!("Error removing channel #{}", &channel)).await?;
                            match e {
                                sqlx::Error::Database(e) => {
                                    let code = e.code().unwrap_or(Cow::Borrowed(""));
                                    msg.reply(ctx, format!("Error removing channel #{}: `{}`", channel, code)).await?;
                                    // if code == "2067" { // SQLITE_CONSTRAINT_UNIQUE (UNIQUE constraint failed)
                                    //     msg.reply(ctx, format!("Channel {} is not in the list", &channel)).await?;
                                    // } else {
                                    //     // msg.reply(ctx, format!("Error removing channel #{}: {}", channel, e.message())).await?;
                                    //     msg.reply(ctx, format!("Error removing channel #{}", &channel)).await?;
                                    // }
                                },
                                _ => {
                                    // msg.reply(ctx, format!("Error removing channel {}: {:?}", &channel, e)).await?;
                                    msg.reply(ctx, format!("Error removing channel #{}", channel)).await?;
                                }
                            }
                        }
                    }
                    match tx.commit().await {
                        Ok(_) => { msg.reply(ctx, "Removed channels").await?; },
                        // TODO: Make so no data leaks through the error message
                        Err(e) => { msg.reply(ctx, format!("Error removing channels: {:?}", e)).await?; },
                    }
                },
                Actions::List => {
                    let rows = sqlx::query("SELECT channel FROM channels WHERE discord_user_id = ?")
                        .bind(msg.author.id.0 as i64)
                        .fetch_all(db_con).await?;

                    let mut channels = rows.iter().map(|row|
                        format!("#{}", row.try_get::<String, &str>("channel")
                            .expect("SQL query returned invalid data")
                        )
                    ).collect::<Vec<_>>();
                    channels.sort();

                    msg.channel_id.send_message(ctx, |m| {
                        m.embed(|e| {
                            e.title("Monitored channels");
                            e.description(channels.join(", "));
                            e
                        });
                        m
                    }).await?;
                },
            }
        },
        Err(e) => {
            // msg.reply(ctx, format!("Error parsing command: {}", e)).await?;
            msg.reply(ctx, styled_str::fmt_args_error(&e)).await?;
        }
    }

    Ok(())
}
