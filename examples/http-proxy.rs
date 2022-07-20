use futures_util::future;
use twilight_http::Client;
use twilight_model::id::Id;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the tracing subscriber.
    tracing_subscriber::fmt::init();

    let client = Client::builder()
        .proxy("localhost:3000".to_owned(), true)
        .ratelimiter(None)
        .build();
    let channel_id = Id::new(620_980_184_606_048_278);

    future::join_all((1u8..=10).map(|x| {
        client
            .create_message(channel_id)
            .content(&format!("Ping #{x}"))
            .expect("content not a valid length")
            .exec()
    }))
    .await;

    let me = client.current_user().exec().await?.model().await?;
    println!("Current user: {}#{}", me.name, me.discriminator);

    Ok(())
}
