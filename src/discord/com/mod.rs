
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
        let mut $var = $var.get_mut();
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
