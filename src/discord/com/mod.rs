
mod general;
mod channel;
mod trigger;

pub use general::GENERAL_GROUP;
pub use channel::CHANNEL_GROUP;
pub use trigger::TRIGGER_GROUP;


macro_rules! get_db {
    ($ctx:expr, $var:ident) => {
        let mut $var = $ctx.data.write().await;
        let $var = $var.get_mut::<DbConnection>().unwrap();
        let $var = $var.get_mut();
    };
}
pub(crate) use get_db;

macro_rules! get_bot_prefix {
    ($ctx:expr) => {
        {
            let data = $ctx.data.read().await;
            data.get::<CommandPrefix>().unwrap().clone()
        }
    };
}
pub(crate) use get_bot_prefix;

macro_rules! update_channel_count {
    ($ctx:expr, $delta:expr) => {
        {
            let prefix = get_bot_prefix!($ctx);
            let channel_count = {
                let data = $ctx.data.read().await;
                data.get::<ChannelCount>().unwrap().clone()
            };
            let channel_count = channel_count + $delta;

            {
                let mut data = $ctx.data.write().await;
                data.insert::<ChannelCount>(channel_count);
            }

            $ctx.set_activity(Activity::watching(format!("{} chats | DM {}help", channel_count, prefix))).await;
        }
    };
}
pub(crate) use update_channel_count;
