use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::{command, group};

use crate::discord::com::get_bot_prefix;

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

    let prefix = get_bot_prefix!(ctx);

    macro_rules! cmd {
        ($com:expr, $des:expr) => {
            format!("`{}{}`\t - \t{}\n", prefix, $com, $des).as_str()
        }
    }
    macro_rules! cmd_list {
        ($($com:expr),*) => {
            {
                let mut s = String::new();
                $( s.push_str($com); )*
                // s.replace("\t", "⠀") // Blank braille character
                s
            }
        };
    }

    // TODO: Rework this to use clap? Maybe when clap updates?
    msg.channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.title("Help");
            e.description("Me when help lol\n\nWhy is this text styled so bad? I don't know, I'm not a designer.\nDiscord eats whitespaces and I don't want to make the whole thing a code block :shrug:");
            e.fields(vec![
                ("General",
                 cmd_list!(
                     cmd!("ping", "Pong!"),
                     cmd!("help", "Print this help message"),
                     cmd!("about", "About this bot")
                 ), false),
                ("Channel", cmd_list!(
                     cmd!("channel add <channels>", "Add channels to watchlist"),
                     cmd!("channel remove <channels>", "Remove channels from watchlist"),
                     cmd!("channel list", "List all channels in watchlist")
                 ), false),
                ("Trigger", cmd_list!(
                     cmd!("trigger add <trigger>", "Add plaintext match trigger (ex: \"AzureDiamond\")"),
                    "`\t-r, --regex`\tAdd regex match trigger (ex: *todo*)\n",
                    "`\t           `\tThe regex flavour is Rust, see [here](https://docs.rs/regex/latest/regex/#syntax)\n",
                    "`\t-c, --case-sensitive`\tMatch case-sensitive (default: case-insensitive)\n",
                     cmd!("trigger remove <ids>", "Remove triggers with specified ids"),
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
