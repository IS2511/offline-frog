use irc::client::prelude::*;
use thiserror::Error;
use ahash::AHashMap;
use irc::client::ClientStream;

use crate::TriggerEvent;



#[derive(Debug)]
pub enum IrcMessageEvent {
    Incoming(Message),
    Outgoing(Command),
}

pub fn make_join_msg(channel: String) -> IrcMessageEvent {
    // IrcMessageEvent::Outgoing(Message::from(Command::JOIN(channel, None, None)))
    IrcMessageEvent::Outgoing(Command::JOIN(format!("#{}", channel), None, None))
}

pub fn make_part_msg(channel: String) -> IrcMessageEvent {
    IrcMessageEvent::Outgoing(Command::PART(format!("#{}", channel), None))
}


#[derive(Debug, Clone)]
pub struct TwitchMessageSimple {
    pub channel: String,
    pub author: String,
    pub message: String,
    pub triggers: Vec<(u16, u16)>,
}

impl TwitchMessageSimple {
    pub fn new(channel: String, author: String, message: String) -> Self {
        Self {
            channel,
            author,
            message,
            triggers: Vec::new(),
        }
    }

    pub fn add_trigger(&mut self, trig: (u16, u16)) {
        // Insert the trigger sorted by start position
        let len_before = self.triggers.len();
        for (i, t) in self.triggers.iter().enumerate() {
            if trig.0 < t.0 {
                self.triggers.insert(i, trig);
                break;
            }
        }
        if self.triggers.len() == len_before {
            self.triggers.push(trig);
        }
        self.normalize_triggers();
    }

    pub fn message_highlighted(&self, highlighter: &str) -> String {
        // TODO: More consistent highlights, this bonks when multiple triggers in one message
        let mut message = self.message.clone();
        // Since triggers should be sorted by start position, we can just reverse the iterator
        for (start, end) in self.triggers.iter().rev() {
            let start = *start as usize;
            let end = *end as usize;
            let with_highlight = format!("{}{}{}", highlighter, &message[start..end], highlighter);
            message.replace_range(start..end, &with_highlight);
        }
        message
    }

    fn normalize_triggers(&mut self) {
        // Normalize triggers to be in order and not overlapping
        // NOTE: We assume triggers are already sorted by `start` (sorted insert)
        // Should be O(n), only one pass. Memory is not that good though...
        let mut new_triggers = Vec::new();
        let mut last_end = 0;
        for (start, end) in self.triggers.iter() {
            if *start >= last_end {
                new_triggers.push((*start, *end));
                last_end = *end;
            } else if *end > last_end {
                let len = new_triggers.len();
                new_triggers.get_mut(len - 1).unwrap().1 = *end;
                last_end = *end;
            }
        }
        self.triggers = new_triggers;
    }
}


#[derive(Debug, Error)]
pub enum IrcThreadError {
    #[error("IRC error: {0}")]
    IrcError(#[from] irc::error::Error),
    #[error("SQL error: {0}")]
    SqlxError(#[from] sqlx::Error),
    // #[error("mpsc channel error: {0}")]
    // ChannelError(#[from] tokio::sync::mpsc::error::SendError<>),
}

pub struct TwitchClient {
    client: Client,
    config: Config,
    db_con: tokio::sync::Mutex<sqlx::pool::PoolConnection<sqlx::Sqlite>>,
    discord_tx: tokio::sync::mpsc::Sender<TriggerEvent>,
}

pub async fn make_client(mut db_con: sqlx::pool::PoolConnection<sqlx::Sqlite>, tx: tokio::sync::mpsc::Sender<TriggerEvent>) -> Result<TwitchClient, irc::error::Error> {
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
        server: Some("irc.chat.twitch.tv".to_string()),
        port: Some(6697),
        channels: sqlx::query!("SELECT DISTINCT channel FROM channels")
            .fetch_all(&mut db_con)
            .await
            .expect("Failed to fetch channels from DB")
            .into_iter()
            .map(|row| format!("#{}", row.channel))
            .collect::<Vec<String>>(),
        ..Config::default()
    };

    let client = Client::from_config(config.clone()).await?;
    client.identify()?;
    // client.send(Command::CAP(
    //     None,
    //     command::CapSubCommand::REQ,
    //     None,
    //     Some("twitch.tv/membership".to_string())))?;

    Ok(TwitchClient {
        client,
        config,
        db_con: tokio::sync::Mutex::new(db_con),
        discord_tx: tx,
    })
}

impl TwitchClient {

    pub fn send(&self, message: impl Into<Message>) -> Result<(), irc::error::Error> {
        self.client.send(message)
    }

    pub fn stream(&mut self) -> Result<ClientStream, irc::error::Error> {
        self.client.stream()
    }

    pub async fn handle(&mut self, message: &Message) -> Result<(), IrcThreadError> {
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
                // irc_debug!("{} says to {}: {}", author_nickname, target, msg);

                let channel_name = target.strip_prefix('#').unwrap_or(target).to_lowercase();

                let msg_template = TwitchMessageSimple::new(
                    channel_name.clone(),
                    author_nickname.to_string(),
                    msg.to_string()
                );
                // irc_debug!("msg_template: {:?}", msg_template);

                let mut messages_per_user = AHashMap::new();

                macro_rules! append_trigger {
                        ($user:expr, $trig:expr) => {
                            if !messages_per_user.contains_key($user) {
                                messages_per_user.insert($user.clone(), msg_template.clone());
                            }
                            messages_per_user.get_mut($user).unwrap().add_trigger($trig);
                        };
                    }


                let query = sqlx::query_as!(crate::db::TriggerRecordNoId,
                        "SELECT discord_user_id, trigger, case_sensitive, regex FROM triggers WHERE discord_user_id IN (SELECT discord_user_id FROM channels WHERE channel = ?)",
                        channel_name);

                let res = query.fetch_all(self.db_con.get_mut()).await;
                if res.is_err() {
                    irc_debug!("Error fetching triggers");
                }
                let triggers = res?;

                for row in triggers {
                    let discord_id = row.discord_user_id;
                    let trigger = row.trigger;
                    let case_sensitive = row.case_sensitive;
                    let regex = row.regex;

                    // irc_debug!("Got trigger: `{}` discord {}", trigger, discord_id);

                    if regex {
                        // TODO: Make sure regex in DB is valid (check when putting in)
                        if case_sensitive {
                            let re = regex::Regex::new(&trigger).unwrap();
                            for mat in re.find_iter(msg) {
                                append_trigger!(&discord_id, (mat.start() as u16, mat.end() as u16));
                            }
                        } else {
                            let re = regex::Regex::new(format!("(?i:{})", trigger).as_str()).unwrap();
                            for mat in re.find_iter(msg) {
                                append_trigger!(&discord_id, (mat.start() as u16, mat.end() as u16));
                            }
                        }
                    } else if case_sensitive {
                        for pos in msg.match_indices(&trigger) {
                            append_trigger!(&discord_id, (pos.0 as u16, (pos.0 + trigger.len()) as u16));
                        }
                    } else {
                        for pos in msg.to_lowercase().match_indices(&trigger.to_lowercase()) {
                            append_trigger!(&discord_id, (pos.0 as u16, (pos.0 + trigger.len()) as u16));
                        }
                    }
                }

                for (discord_id, msg) in messages_per_user {
                    println!("Sending message to discord: {:?}", msg);
                    self.discord_tx.send(TriggerEvent::new(
                        discord_id as u64,
                        msg,
                        chrono::Utc::now()
                    )).await.unwrap_or_else(|e| {
                        println!("ERROR! Too many events in queue, failed to add: {:?}", e);
                    });
                }
            }
            Command::JOIN(ref _channels, ref _chan_keys,  ref _real_name) => {
                // irc_debug!("{} ({:?}) joined {}", author_nickname, real_name, channels);
                if author_nickname == self.client.current_nickname() {
                    // Just successfully joined a channel, report back?
                }
            }
            // Command::PART(ref channels, ref comment) => {
            //     irc_debug!("{} left {} ({:?})", author_nickname, channels, comment);
            // }
            // Command::QUIT(ref comment) => {
            //     irc_debug!("{} quit ({:?})", author_nickname, comment);
            // }
            // Command::ERROR(ref msg) => {
            //     irc_debug!("error: {}", msg);
            // }
            Command::Response(ref response, ref args) => {
                match response {
                    Response::RPL_WELCOME => {
                        // irc_debug!("Connected to IRC");
                        println!("`{}` connected!", args[0]);
                    }
                    Response::RPL_NAMREPLY => {
                        // irc_debug!("{:?}", args);
                    }
                    _ => {
                        // irc_debug!("Response: {:?}", response);
                    }
                }
            }
            Command::Raw(ref code, ref args) => {
                irc_debug!("raw: {} {:?}", code, args);
                // TODO: Handle twitch-specific commands (ex: RECONNECT)
                match code.as_str() {
                    "RECONNECT" => {
                        let mut config = self.config.clone();
                        config.channels = self.client.list_channels().unwrap();
                        self.client = Client::from_config(config).await?;
                    }
                    _ => {}
                };
            }
            _ => {
                // irc_debug!("unhandled: {:?}", message);
            }
        }
        Ok(())
    }

}
