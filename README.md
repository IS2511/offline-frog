# Offline-frog

> [!WARNING]
> This project is no longer maintained.
> The public instance has been retired and the code may be outdated.

Offline-frog is a simple discord bot for trigger-based notification from twitch chat.
The bot operates entirely in DMs, no privileged intents required.

Main components:
- [serenity](https://crates.io/crates/serenity) for discord
- [irc](https://crates.io/crates/irc) for twitch chat
- [sqlx](https://crates.io/crates/sqlx) (SQLite) for user settings storage

## Usage (public instance)

The public instance has been retired.

<details>
<summary>Old info (spoiler)</summary>

| Offline Frog |
|:------------:|
| ![Offline Frog avatar](https://cdn.discordapp.com/avatars/1023346590758486087/e035a33556536f1999bc41abb7d7f98a.png?size=64) |

- Discord tag: `Offline Frog#2640`

You can [add the bot to your server](https://discord.com/api/oauth2/authorize?client_id=760000000000000000&permissions=0&scope=bot).
The bot will have no permissions and will not be able to do anything on your server.
It's there so users of your server can DM it.

After you have a way to DM the bot, send `!help` to get started.

</details>

## Usage (self-hosted)

1. Create a discord bot and invite it to your server: https://discordapp.com/developers/applications/
2. Create a `.env` file and fill it with the following mandatory variables:
```
DISCORD_TOKEN=your_discord_token
DATABASE_URL=your_url_here # example: `sqlite:local.sqlite?mode=rwc`
```
3. Build with `cargo build --release`
4. Run with `./target/release/offline-frog`
