use std::borrow::Cow;
use std::fmt::Write as _; // import without risk of name clashing
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::{command, group};

use clap::{Parser, Subcommand};
use sqlx::{Acquire};

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
        // TODO: Split whitespace but preserver quoted substrings
        // TODO: Reverse-parse discord styling things? Accepts `channel`?
        //  Somehow allow stuff like \_ because it's what looks valid in discord
        // TODO: Construct regex and check validity before adding to db

        /// Trigger to add (either plain text or regex pattern)
        // #[arg(num_args = 1..)]
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

    let author_id = msg.author.id.0 as i64;

    match args {
        Ok(args) => {
            match args.action {
                Actions::Add { trigger, case_sensitive, regex } => {
                    let trigger = match case_sensitive {
                        true => trigger,
                        false => trigger.to_lowercase(),
                    };

                    get_db!(ctx, db);

                    let mut tx = db.begin().await?;
                    let res = sqlx::query!("INSERT INTO triggers (discord_user_id, trigger, case_sensitive, regex) VALUES (?, ?, ?, ?)",
                        author_id,
                        trigger,
                        case_sensitive,
                        regex)
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
                    let triggers = {
                        get_db!(ctx, db);

                        let triggers = sqlx::query!("SELECT id FROM triggers WHERE discord_user_id = ?",
                        author_id)
                            .fetch_all(db)
                            .await;
                        if triggers.is_err() {
                            msg.reply(ctx, "Failed to get triggers".to_string()).await?;
                            return Ok(());
                        }
                        let triggers = triggers.unwrap();
                        let mut triggers: Vec<i64> = triggers.iter().map(|i| i.id).collect();
                        triggers.sort();
                        triggers
                    };

                    get_db!(ctx, db);

                    let mut tx = db.begin().await?;
                    let mut failed_ids = Vec::new();
                    let triggers_len = triggers.len() as i64;
                    for id in &ids {
                        if (id > &triggers_len) || (id < &1) {
                            failed_ids.push(id.to_string());
                            break;
                        }
                        let id = (id - 1) as usize;
                        let trigger_id = triggers[id] as i64;
                        let res = sqlx::query!("DELETE FROM triggers WHERE discord_user_id = ? AND id = ?",
                            author_id,
                            trigger_id)
                            .execute(&mut tx)
                            .await;
                        if res.is_err() {
                            failed_ids.push(id.to_string());
                            break;
                        }
                    }
                    if failed_ids.is_empty() {
                        tx.commit().await?;
                        msg.reply(ctx, format!("Removed {} triggers", ids.len())).await?;
                    } else {
                        tx.rollback().await?;
                        // msg.reply(ctx, format!("Failed to remove triggers: **{}**", failed_ids.join(", "))).await?;
                        msg.reply(ctx, format!("Failed to remove trigger: **{}**. Rollback.", failed_ids[0])).await?;
                    }
                },
                Actions::List => {
                    get_db!(ctx, db);

                    let res = sqlx::query_as!(crate::db::TriggerRecordNoDiscord,
                        "SELECT id, trigger, case_sensitive, regex FROM triggers WHERE discord_user_id = ?",
                        author_id)
                        .fetch_all(db)
                        .await;
                    if res.is_err() {
                        msg.reply(ctx, "Failed to list triggers".to_string()).await?;
                        return Ok(());
                    }
                    let mut res = res.unwrap();
                    res.sort_by(|a, b| a.id.cmp(&b.id));
                    let mut reply = String::new();
                    let mut i = 1;
                    for row in res {
                        use crate::discord::extra::IntoEmoji;
                        let _ = writeln!(reply, "**ID {}**: `{}` (case_sensitive: {}, regex: {})",
                                 i, row.trigger, row.case_sensitive.emoji(), row.regex.emoji());
                        i += 1;
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
