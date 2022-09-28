use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::{command, group};

use crate::discord::CommandPrefix;

#[group]
#[commands(ping, help, about)]
struct General;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}


#[command]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {

    let prefix = {
        let data = ctx.data.read().await;
        data.get::<CommandPrefix>().unwrap().clone()
    };

    macro_rules! cmd {
        ($com:expr, $des:expr) => {
            format!("`{}{}` - {}\n", prefix, $com, $des)
        }
    }
    macro_rules! cmd_list {
        ($($com:expr),*) => {
            {
                let mut s = String::new();
                $( s.push_str($com.as_str()); )*
                s
            }
        };
    }

    msg.channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.title("Help");
            e.description("Me when help lol");
            e.fields(vec![
                ("General",
                 cmd_list!(
                     cmd!("ping", "Pong!"),
                     cmd!("help", "Print this help message"),
                     cmd!("about", "About this bot")
                 ), false),
                ("Channel", cmd_list!(
                     cmd!("channel add <channel>", "Add channel to watchlist"),
                     cmd!("channel remove <channel>", "Remove channel from watchlist"),
                     cmd!("channel list", "List all channels in watchlist")
                 ), false),
                ("Trigger", cmd_list!(
                     cmd!("trigger add <trigger>", "Add plaintext match trigger (ex: your username)"),
                     cmd!("trigger remove <id>", "Remove trigger with specified id"),
                     cmd!("trigger list", "List all triggers and their ids")
                 ), false),
            ]);
            e
        });
        m
    }).await?;

    Ok(())
}

#[command]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.title("About");
            e.description(concat!(
                "Frogs are cool!\n",
                "This bot was made by IS2511.\n",
                "It's purpose is to relay twitch chat mentions to discord DMs."
            ));
            e
        });
        m
    }).await?;

    Ok(())
}
