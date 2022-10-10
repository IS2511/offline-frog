use std::borrow::Cow;
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::{command, group};

use clap::{Parser, Subcommand};
use sqlx::{Acquire};

use crate::discord::{CommandPrefix, DbConnection, styled_str};
use crate::discord::com::{get_bot_prefix, get_db};
use crate::discord::styled_str::escape_twitch_channel;

/// Arguments to the ignore command
#[derive(clap::Parser, Debug)]
struct Args {
    /// Action to perform
    #[command(subcommand)]
    action: Actions,
}

#[derive(Subcommand, Debug)]
enum Actions {
    /// Add usernames to the list of ignored users
    Add {
        /// Usernames to add
        usernames: Vec<String>,
    },
    /// Remove usernames from the list of ignored users
    Remove {
        /// Usernames to remove
        usernames: Vec<String>,
    },
    /// List all usernames of ignored users
    List,
}

#[group]
#[commands(ignore)]
struct Ignore;

#[command]
async fn ignore(ctx: &Context, msg: &Message) -> CommandResult {
    let prefix = get_bot_prefix!(ctx);

    let args = Args::try_parse_from(msg.content.trim_start_matches(&prefix).split_whitespace());

    let author_id = msg.author.id.0 as i64;

    match args {
        Ok(args) => {
            match args.action {
                Actions::Add { usernames } => {
                    let usernames = usernames.iter().map(|c| c.to_lowercase()).collect::<Vec<_>>();

                    get_db!(ctx, db);

                    let mut tx = db.begin().await?;
                    for username in &usernames {
                        let res = sqlx::query!("INSERT OR IGNORE INTO ignores (discord_user_id, username) VALUES (?, ?)",
                            author_id,
                            username)
                            .execute(&mut tx).await;
                        if let Err(e) = res {
                            match e {
                                sqlx::Error::Database(e) => {
                                    let code = e.code().unwrap_or(Cow::Borrowed(""));
                                    if code == "2067" { // SQLITE_CONSTRAINT_UNIQUE (UNIQUE constraint failed)
                                        msg.reply(ctx, format!("Username {} is already in the list", escape_twitch_channel(username))).await?;
                                    } else {
                                        msg.reply(ctx, format!("Error adding username {}", escape_twitch_channel(username))).await?;
                                    }
                                }
                                _ => {
                                    msg.reply(ctx, format!("Error adding username {}", escape_twitch_channel(username))).await?;
                                }
                            }
                        }
                    }
                    match tx.commit().await {
                        Ok(_) => { msg.reply(ctx, "Added channels").await?; }
                        // TODO: Make so no data leaks through the error message
                        Err(e) => { msg.reply(ctx, format!("Error adding usernames: {:?}", e)).await?; }
                    }

                },
                Actions::Remove { usernames } => {
                    let usernames = usernames.iter().map(|c| c.to_lowercase()).collect::<Vec<_>>();

                    get_db!(ctx, db);

                    let mut tx = db.begin().await?;
                    for username in &usernames {
                        let res = sqlx::query!("DELETE FROM ignores WHERE discord_user_id = ? AND username = ?",
                            author_id,
                            username)
                            .execute(&mut tx).await;
                        if let Err(e) = res {
                            match e {
                                sqlx::Error::Database(e) => {
                                    let code = e.code().unwrap_or(Cow::Borrowed(""));
                                    msg.reply(ctx, format!("Error removing username {}: `{}`", escape_twitch_channel(username), code)).await?;
                                }
                                _ => {
                                    msg.reply(ctx, format!("Error removing username {}", escape_twitch_channel(username))).await?;
                                }
                            }
                        }
                    }
                    match tx.commit().await {
                        Ok(_) => { msg.reply(ctx, "Removed usernames").await?; }
                        // TODO: Make so no data leaks through the error message
                        Err(e) => { msg.reply(ctx, format!("Error removing usernames: {:?}", e)).await?; }
                    }

                },
                Actions::List => {
                    get_db!(ctx, db);

                    let rows = sqlx::query!("SELECT username FROM ignores WHERE discord_user_id = ?",
                        author_id)
                        .fetch_all(db).await?;

                    let mut usernames = rows.iter().map(|row| escape_twitch_channel(&row.username)).collect::<Vec<_>>();
                    usernames.sort();

                    msg.channel_id.send_message(ctx, |m| {
                        m.embed(|e| {
                            e.title("Ignored users");
                            e.description(usernames.join(", "));
                            e
                        });
                        m
                    }).await?;
                },
            }
        },
        Err(e) => {
            msg.reply(ctx, styled_str::fmt_args_error(&e)).await?;
        }
    }

    Ok(())
}
