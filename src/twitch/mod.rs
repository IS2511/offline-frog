use irc::client::prelude::*;
use futures_util::{StreamExt, TryStreamExt};
use sqlx::{Executor, Row};
// use irc::proto::command;

// use crate::KvStore;

pub async fn start(db_con: sqlx::pool::PoolConnection<sqlx::Sqlite>) {
    if let Err(why) = start_client(db_con).await {
        println!("[IRC] An error occurred while running the client: {:?}", why);
    }
}

pub async fn start_client(db_con: sqlx::pool::PoolConnection<sqlx::Sqlite>) -> Result<(), irc::error::Error> {
    // We can also load the Config at runtime via Config::load("path/to/config.toml")
    let config = Config {
        nickname: Some(format!("justinfan{}", rand::random::<u32>())),
        alt_nicks: vec![ // Just in case the first one is taken
            format!("justinfan{}", rand::random::<u32>()),
            format!("justinfan{}", rand::random::<u32>()),
            format!("justinfan{}", rand::random::<u32>()),
        ],
        // realname: Some("Offline_Frog".to_string()),
        use_tls: Some(true),
        server: Some("irc.chat.twitch.tv".to_owned()),
        port: Some(6697),
        channels: vec!["#offline_frog".to_owned()],
        ..Config::default()
    };

    let mut client = Client::from_config(config).await?;
    client.identify()?;
    // client.send(Command::CAP(
    //     None,
    //     command::CapSubCommand::REQ,
    //     None,
    //     Some("twitch.tv/membership".to_string())))?;

    let mut stream = client.stream()?;

    while let Some(message) = stream.next().await.transpose()? {
        let author_nickname = message.source_nickname().unwrap_or("");

        macro_rules! irc_debug {
            ($format:expr, $($arg:expr),+) => {
                println!("[IRC] {}", format!($format, $($arg),+));
            };
            ($format:expr) => {
                println!("[IRC] {}", $format);
            };
        }

        match message.command {
            Command::PRIVMSG(ref target, ref msg) => {
                irc_debug!("{} says to {}: {}", author_nickname, target, msg);

                let channel_name = target.strip_prefix('#').unwrap_or(target);

                // TODO: Get all the triggers for the channel (with discord ids)

                let query = sqlx::query("SELECT discord_user_id, trigger, case_sensitive, regex FROM triggers WHERE discord_user_id IN (SELECT discord_user_id FROM channels WHERE channel = ?)")
                    .bind(channel_name);

                let mut triggers = db_con.fetch(query);

                for row in triggers.next().await? {
                    let discord_id = row.try_get::<String, &str>("discord_user_id")?;
                    let trigger = row.try_get::<String, &str>("trigger")?;
                    let case_sensitive = row.try_get::<bool, &str>("case_sensitive")?;
                    let regex = row.try_get::<bool, &str>("regex")?;

                    if regex {
                        irc_debug!("Not yet implemented regex! trigger: {}", trigger);
                    } else {
                        if case_sensitive {
                            if msg.contains(&trigger) {

                            }
                        } else {
                            if msg.to_lowercase().contains(&trigger.to_lowercase()) {

                            }
                        }
                    }
                }

                // db_con.fetch(sqlx::query!())
            }
            Command::JOIN(ref channels, ref _chan_keys,  ref real_name) => {
                irc_debug!("{} ({:?}) joined {}", author_nickname, real_name, channels);
            }
            Command::PART(ref channels, ref comment) => {
                irc_debug!("{} left {} ({:?})", author_nickname, channels, comment);
            }
            Command::QUIT(ref comment) => {
                irc_debug!("{} quit ({:?})", author_nickname, comment);
            }
            Command::ERROR(ref msg) => {
                irc_debug!("error: {}", msg);
            }
            Command::Raw(ref code, ref args) => {
                irc_debug!("raw: {} {:?}", code, args);
                // TODO: Handle twitch-specific commands (ex: RECONNECT)
            }
            _ => {
                irc_debug!("unhandled: {:?}", message);
            }
        }

    }

    Ok(())
}
