use std::borrow::Cow;
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::{command, group};

use clap::{Parser, Subcommand};
use sqlx::{Acquire, Row};

use crate::discord::{CommandPrefix, DbConnection, styled_str};
use crate::discord::com::{get_bot_prefix, get_db};


/// Arguments to the trigger command
#[derive(clap::Parser, Debug)]
struct Args {
    /// Action to perform
    #[command(subcommand)]
    action: Actions,
}

#[derive(Subcommand, Debug)]
enum Actions {
    /// Add a trigger to the list of triggers
    Add {
        /// Trigger to add (either plain text or regex pattern)
        trigger: String,

        /// Take case into account when matching
        #[arg(short, long, default_value_t = false)]
        case_sensitive: bool,

        /// Use regex pattern matching (regex)
        #[arg(short, long, default_value_t = false)]
        regex: bool,
    },
    /// Remove triggers from the list of triggers
    Remove {
        /// IDs of the triggers to remove
        ids: Vec<i64>,
    },
    /// List all triggers
    List,
}

#[group]
#[commands(trigger)]
struct Trigger;

#[command]
async fn trigger(ctx: &Context, msg: &Message) -> CommandResult {
    let prefix = get_bot_prefix!(ctx);

    let args = Args::try_parse_from(msg.content.trim_start_matches(&prefix).split_whitespace());

    get_db!(ctx, db_con);

    match args {
        Ok(args) => {
            match args.action {
                Actions::Add { trigger, case_sensitive, regex } => {
                    let mut tx = db_con.begin().await?;
                    let res = sqlx::query("INSERT INTO triggers (discord_user_id, trigger, case_sensitive, regex) VALUES (?, ?, ?, ?)")
                        .bind(msg.author.id.0 as i64)
                        .bind(&trigger)
                        .bind(case_sensitive)
                        .bind(regex)
                        .execute(&mut tx)
                        .await;
                    if let Err(e) = res {
                        tx.rollback().await?;
                        match e {
                            sqlx::Error::Database(e) => {
                                if e.code() == Some(Cow::Borrowed("2067")) { // SQLITE_CONSTRAINT_UNIQUE (UNIQUE constraint failed)
                                    msg.reply(ctx, "Trigger already exists").await?;
                                } else {
                                    msg.reply(ctx, "Failed to add trigger").await?;
                                }
                            },
                            _ => {
                                msg.reply(ctx, "Failed to add trigger").await?;
                            }
                        }
                        // msg.reply(ctx, format!("Failed to add trigger: {}", e)).await?;
                    } else {
                        tx.commit().await?;
                        msg.reply(ctx, format!("Added trigger: `{}`", trigger)).await?;
                    }
                }
                Actions::Remove { ids } => {
                    let mut tx = db_con.begin().await?;
                    let mut failed_list = Vec::new();
                    for id in &ids {
                        let res = sqlx::query("DELETE FROM triggers WHERE discord_user_id = ? AND id = ?")
                            .bind(msg.author.id.0 as i64)
                            .bind(id)
                            .execute(&mut tx)
                            .await;
                        if let Err(e) = res {
                            failed_list.push(id.to_string());
                        }
                    }
                    if failed_list.is_empty() {
                        tx.commit().await?;
                        msg.reply(ctx, format!("Removed {} triggers", ids.len())).await?;
                    } else {
                        tx.rollback().await?;
                        // msg.reply(ctx, format!("Failed to remove triggers: `{}`", failed_list.join(", "))).await?;
                        msg.reply(ctx, format!("Failed to remove trigger: `{}`", failed_list[0])).await?;
                    }
                },
                Actions::List => {
                    let res = sqlx::query("SELECT id, trigger, case_sensitive, regex FROM triggers WHERE discord_user_id = ?")
                        .bind(msg.author.id.0 as i64)
                        .fetch_all(db_con)
                        .await;
                    if let Err(e) = res {
                        msg.reply(ctx, format!("Failed to list triggers")).await?;
                        return Ok(());
                    }
                    let res = res.unwrap();
                    let mut reply = String::new();
                    for row in res {
                        let id: i64 = row.get(0);
                        let trigger: String = row.get(1);
                        let case_sensitive: bool = row.get(2);
                        let regex: bool = row.get(3);
                        // TODO: Escape discord styling in `trigger` before printing
                        reply.push_str(&format!("**{}**: `{}` (case_sensitive: {}, regex: {})\n", id, trigger, case_sensitive, regex));
                    }
                    msg.channel_id.send_message(ctx, |m|
                        m.embed(|e|
                            e.title("Triggers")
                            .description(reply)
                        )
                    ).await?;
                },
            }
        },
        Err(e) => {
            msg.reply(ctx, styled_str::fmt_args_error(&e)).await?;
        },
    }

    Ok(())
}
