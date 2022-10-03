// use clap::builder::StyledStr;

// struct StyledStrBasic {
//     pub pieces: Vec<(Option<Style>, String)>,
// }
//
// enum Style {
//     Bold,
//     Italic,
//     Underline,
//     Strikethrough,
//     Monospace,
//     Spoiler,
// }

// // Maybe like this, with nesting types to apply multiple styles at once?
// enum StyledFragment {
//     Bold(Option<Style>, String),
// }
//
// // Or like this?
// enum StyleNested {
//     Bold(Option<StyleNested>),
// }

// impl From<StyledStr> for StyledStrBasic {
//     fn from(styled_str: StyledStr) -> StyledStrBasic {
//         // DEPRECATED: StyledStr -> String (ANSI) -> StyledStrBasic -> DiscordDisplay -> String (Discord)
//         // TODO: Wait for clap to implement a better way to get style info out of StyledStr
//         let ansi = styled_str.ansi();
//         StyledStrBasic {
//             pieces: Vec::new(),
//         }
//     }
// }

// Copy of `clap::builder::Style`
// enum Style {
//     Header,
//     Literal,
//     Placeholder,
//     Good,
//     Warning,
//     Error,
//     Hint,
// }

// impl DiscordDisplay {
//     fn new(inner: StyledStr) -> Self {
//         Self { inner }
//     }
// }

// impl std::fmt::Display for DiscordDisplay {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         for (_, content) in self.inner.iter() {
//             // ok!(std::fmt::Display::fmt(content, f));
//         }
//         let mut message = self.inner.to_string();
//         for (start, end) in self.inner.triggers.iter() {
//             let start = *start as usize;
//             let end = *end as usize;
//             let with_highlight = format!("{}{}{}", highlighter, &message[start..end], highlighter);
//             message.replace_range(start..end, &with_highlight);
//         }
//         write!(f, "{}", message)
//     }
// }

pub fn fmt_args_error(e: &clap::Error) -> String {
    format!("*Invalid command or arguments*\n{}", e.render())
}

// TODO: Escaping and un-escaping discord-flavored markdown

pub fn escape_twitch_channel(channel: &str) -> String {
    channel.replace('_', "\\_")
}

pub fn unescape_twitch_channel(channel: &str) -> String {
    channel.replace('_', "\\_")
}

// pub fn escape_markdown(s: &str) -> String {
//     s.replace('*', "\\*")
//         .replace('_', "\\_")
//         .replace('~', "\\~")
//         .replace('`', "\\`")
// }
