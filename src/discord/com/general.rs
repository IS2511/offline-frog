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
            e.description("Basic flow:\n1) Add channels you are interested in\n2) Add at least one trigger (ex: your twitch username)\n3) Get a DM when someone says your trigger in a channel you are monitoring\n\nGet pinged <:tf:1027007493764821083>");
            e.fields(vec![
                ("General",
                 cmd_list!(
                     cmd!("ping", "Pong!"),
                     cmd!("help", "Print this help message")
                     // cmd!("about", "About this bot")
                 ), false),
                ("Channel", cmd_list!(
                     cmd!("channel add <channels>", "Add channels to watchlist"),
                     cmd!("channel remove <channels>", "Remove channels from watchlist"),
                     cmd!("channel list", "List all channels in watchlist")
                 ), false),
                ("Trigger", cmd_list!(
                     // cmd!("trigger add `\\``<trigger>`\\`` ", "Add plaintext match trigger (ex: \"AzureDiamond\")"),
                     format!("```{}trigger add `<trigger>`\n```Add plaintext match trigger (ex: \"AzureDiamond\")\n", prefix).as_str(),
                    "`\t-r, --regex`\tAdd regex match trigger (ex: \"@is$|@is\\s\")\n",
                    "`\t           `\tThe regex flavour is Rust, see [docs](https://docs.rs/regex/latest/regex/#syntax), test [live](https://rustexp.lpil.uk/)\n",
                    "`\t-c, --case-sensitive`\tMatch case-sensitive (default: case-insensitive)\n\n",
                     cmd!("trigger remove <ids>", "Remove triggers with specified ids"),
                     cmd!("trigger list", "List all triggers and their ids")
                 ), false),
                ("Ignore", cmd_list!(
                     cmd!("ignore add <usernames>", "Add usernames to the list of ignored users"),
                     cmd!("ignore remove <usernames>", "Remove usernames from the list of ignored users"),
                     cmd!("ignore list", "List all usernames of ignored users")
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
