use std::borrow::Cow;
use ahash::AHashSet;
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::{command, group};

use clap::{Parser, Subcommand};
use sqlx::{Acquire};

use crate::discord::{CommandPrefix, DbConnection, IrcEventSender, styled_str};
use crate::discord::com::{get_bot_prefix, get_db};
use crate::twitch::{make_join_msg, make_part_msg};

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

    let irc_tx = {
        let data = ctx.data.read().await;
        data.get::<IrcEventSender>().unwrap().clone()
    };

    get_db!(ctx, db_con);

    let author_id = msg.author.id.0 as i64;

    match args {
        Ok(args) => {
            match args.action {
                Actions::Add { channels } => {
                    let channels = channels.iter().map(|c| c.to_lowercase()).collect::<Vec<_>>();

                    let mut to_be_joined = AHashSet::new();
                    for channel in &channels {
                        let res = sqlx::query!("SELECT EXISTS(SELECT 1 FROM channels WHERE channel = ?) AS result", channel)
                            .fetch_one(&mut *db_con).await?;
                        let exists: bool = res.result == 1;
                        // println!("ADD Channel #{} exists: {}", channel, exists);
                        if !exists {
                            to_be_joined.insert(channel);
                        }
                    }
                    // println!("ADD Channels to be joined: {:?}", to_be_joined);

                    let mut tx = db_con.begin().await?;
                    for channel in &channels {
                        let res = sqlx::query!("INSERT OR IGNORE INTO channels (discord_user_id, channel) VALUES (?, ?)",
                            author_id,
                            channel)
                            .execute(&mut tx).await;
                        if let Err(e) = res {
                            to_be_joined.remove(channel);
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

                    // println!("ADD Channels to be actually joined: {:?}", to_be_joined);
                    for channel in to_be_joined {
                        // msg.reply(ctx, format!("*Fun fact*: Channel #{} wasn't tracked by this bot before, but now is!", channel)).await?;
                        irc_tx.send(make_join_msg(channel.clone())).await?;
                    }
                },
                Actions::Remove { channels } => {
                    let channels = channels.iter().map(|c| c.to_lowercase()).collect::<Vec<_>>();

                    let mut tx = db_con.begin().await?;
                    for channel in &channels {
                        let res = sqlx::query!("DELETE FROM channels WHERE discord_user_id = ? AND channel = ?",
                            author_id,
                            channel)
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

                    for channel in &channels {
                        let res = sqlx::query!("SELECT EXISTS(SELECT 1 FROM channels WHERE channel = ?) AS result", channel)
                            .fetch_one(&mut *db_con).await?;
                        let exists: bool = res.result == 1;
                        // println!("REMOVE Channel #{} exists: {}", channel, exists);
                        if !exists {
                            // msg.reply(ctx, format!("*Fun fact*: Channel #{} no longer needs tracking from this bot!", channel)).await?;
                            irc_tx.send(make_part_msg(channel.clone())).await?;
                        }
                    }
                },
                Actions::List => {
                    let rows = sqlx::query!("SELECT channel FROM channels WHERE discord_user_id = ?",
                        author_id)
                        .fetch_all(db_con).await?;

                    let mut channels = rows.iter().map(|row|
                        format!("#{}", row.channel)
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
