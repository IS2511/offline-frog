use dotenv::dotenv;

mod discord;

#[tokio::main]
async fn main() {
    dotenv().expect("Failed to load .env file");

    // Run discord bot
    discord::start().await;
}

