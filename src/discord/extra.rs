
pub trait IntoEmoji {
    fn emoji(self) -> String;
}

impl IntoEmoji for bool {
    fn emoji(self) -> String {
        if self {
            "✅".to_string()
        } else {
            "❌".to_string()
        }
    }
}
