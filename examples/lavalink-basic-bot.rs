use futures_util::StreamExt;
use hyper::{
    client::{Client as HyperClient, HttpConnector},
    Body, Request,
};
use std::{env, future::Future, net::SocketAddr, str::FromStr, sync::Arc};
use twilight_gateway::{Event, Intents, Shard};
use twilight_http::Client as HttpClient;
use twilight_lavalink::{
    http::LoadedTracks,
    model::{Destroy, Pause, Play, Seek, Stop, Volume},
    Lavalink,
};
use twilight_model::{
    channel::Message,
    gateway::payload::{incoming::MessageCreate, outgoing::UpdateVoiceState},
};
use twilight_standby::Standby;

type State = Arc<StateRef>;

#[derive(Debug)]
struct StateRef {
    http: HttpClient,
    lavalink: Lavalink,
    hyper: HyperClient<HttpConnector>,
    shard: Shard,
    standby: Standby,
}

fn spawn(fut: impl Future<Output = anyhow::Result<()>> + Send + 'static) {
    tokio::spawn(async move {
        if let Err(why) = fut.await {
            tracing::debug!("handler error: {why:?}");
        }
    });
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the tracing subscriber.
    tracing_subscriber::fmt::init();

    let (mut events, state) = {
        let token = env::var("DISCORD_TOKEN")?;
        let lavalink_host = SocketAddr::from_str(&env::var("LAVALINK_HOST")?)?;
        let lavalink_auth = env::var("LAVALINK_AUTHORIZATION")?;
        let shard_count = 1u64;

        let http = HttpClient::new(token.clone());
        let user_id = http.current_user().exec().await?.model().await?.id;

        let lavalink = Lavalink::new(user_id, shard_count);
        lavalink.add(lavalink_host, lavalink_auth).await?;

        let intents = Intents::GUILD_MESSAGES | Intents::GUILD_VOICE_STATES;
        let (shard, events) = Shard::new(token, intents);

        shard.start().await?;

        (
            events,
            Arc::new(StateRef {
                http,
                lavalink,
                hyper: HyperClient::new(),
                shard,
                standby: Standby::new(),
            }),
        )
    };

    while let Some(event) = events.next().await {
        state.standby.process(&event);
        state.lavalink.process(&event).await?;

        if let Event::MessageCreate(msg) = event {
            if msg.guild_id.is_none() || !msg.content.starts_with('!') {
                continue;
            }

            match msg.content.split_once(' ').map(|x| x.0) {
                Some("!join") => spawn(join(msg.0, Arc::clone(&state))),
                Some("!leave") => spawn(leave(msg.0, Arc::clone(&state))),
                Some("!pause") => spawn(pause(msg.0, Arc::clone(&state))),
                Some("!play") => spawn(play(msg.0, Arc::clone(&state))),
                Some("!seek") => spawn(seek(msg.0, Arc::clone(&state))),
                Some("!stop") => spawn(stop(msg.0, Arc::clone(&state))),
                Some("!volume") => spawn(volume(msg.0, Arc::clone(&state))),
                _ => continue,
            }
        }
    }

    Ok(())
}

async fn join(msg: Message, state: State) -> anyhow::Result<()> {
    state
        .http
        .create_message(msg.channel_id)
        .content("What's the channel ID you want me to join?")?
        .exec()
        .await?;

    let author_id = msg.author.id;
    let msg = state
        .standby
        .wait_for_message(msg.channel_id, move |new_msg: &MessageCreate| {
            new_msg.author.id == author_id
        })
        .await?;
    let channel_id = msg.content.parse()?;
    let guild_id = msg.guild_id.expect("known to be present");

    state
        .shard
        .command(&UpdateVoiceState::new(
            guild_id,
            Some(channel_id),
            false,
            false,
        ))
        .await?;

    state
        .http
        .create_message(msg.channel_id)
        .content(&format!("Joined <#{channel_id}>!"))?
        .exec()
        .await?;

    Ok(())
}

async fn leave(msg: Message, state: State) -> anyhow::Result<()> {
    tracing::debug!(
        "leave command in channel {} by {}",
        msg.channel_id,
        msg.author.name
    );

    let guild_id = msg.guild_id.unwrap();
    let player = state.lavalink.player(guild_id).await.unwrap();
    player.send(Destroy::from(guild_id))?;
    state
        .shard
        .command(&UpdateVoiceState::new(guild_id, None, false, false))
        .await?;

    state
        .http
        .create_message(msg.channel_id)
        .content("Left the channel")?
        .exec()
        .await?;

    Ok(())
}

async fn play(msg: Message, state: State) -> anyhow::Result<()> {
    tracing::debug!(
        "play command in channel {} by {}",
        msg.channel_id,
        msg.author.name
    );
    state
        .http
        .create_message(msg.channel_id)
        .content("What's the URL of the audio to play?")?
        .exec()
        .await?;

    let author_id = msg.author.id;
    let msg = state
        .standby
        .wait_for_message(msg.channel_id, move |new_msg: &MessageCreate| {
            new_msg.author.id == author_id
        })
        .await?;
    let guild_id = msg.guild_id.unwrap();

    let player = state.lavalink.player(guild_id).await.unwrap();
    let (parts, body) = twilight_lavalink::http::load_track(
        player.node().config().address,
        &msg.content,
        &player.node().config().authorization,
    )?
    .into_parts();
    let req = Request::from_parts(parts, Body::from(body));
    let res = state.hyper.request(req).await?;
    let response_bytes = hyper::body::to_bytes(res.into_body()).await?;

    let loaded = serde_json::from_slice::<LoadedTracks>(&response_bytes)?;

    if let Some(track) = loaded.tracks.first() {
        player.send(Play::from((guild_id, &track.track)))?;

        let content = format!(
            "Playing **{:?}** by **{:?}**",
            track.info.title, track.info.author
        );
        state
            .http
            .create_message(msg.channel_id)
            .content(&content)?
            .exec()
            .await?;
    } else {
        state
            .http
            .create_message(msg.channel_id)
            .content("Didn't find any results")?
            .exec()
            .await?;
    }

    Ok(())
}

async fn pause(msg: Message, state: State) -> anyhow::Result<()> {
    tracing::debug!(
        "pause command in channel {} by {}",
        msg.channel_id,
        msg.author.name
    );

    let guild_id = msg.guild_id.unwrap();
    let player = state.lavalink.player(guild_id).await.unwrap();
    let paused = player.paused();
    player.send(Pause::from((guild_id, !paused)))?;

    let action = if paused { "Unpaused " } else { "Paused" };

    state
        .http
        .create_message(msg.channel_id)
        .content(&format!("{action} the track"))?
        .exec()
        .await?;

    Ok(())
}

async fn seek(msg: Message, state: State) -> anyhow::Result<()> {
    tracing::debug!(
        "seek command in channel {} by {}",
        msg.channel_id,
        msg.author.name
    );
    state
        .http
        .create_message(msg.channel_id)
        .content("Where in the track do you want to seek to (in seconds)?")?
        .exec()
        .await?;

    let author_id = msg.author.id;
    let msg = state
        .standby
        .wait_for_message(msg.channel_id, move |new_msg: &MessageCreate| {
            new_msg.author.id == author_id
        })
        .await?;
    let guild_id = msg.guild_id.unwrap();
    let position = msg.content.parse::<i64>()?;

    let player = state.lavalink.player(guild_id).await.unwrap();
    player.send(Seek::from((guild_id, position * 1000)))?;

    state
        .http
        .create_message(msg.channel_id)
        .content(&format!("Seeked to {position}s"))?
        .exec()
        .await?;

    Ok(())
}

async fn stop(msg: Message, state: State) -> anyhow::Result<()> {
    tracing::debug!(
        "stop command in channel {} by {}",
        msg.channel_id,
        msg.author.name
    );

    let guild_id = msg.guild_id.unwrap();
    let player = state.lavalink.player(guild_id).await.unwrap();
    player.send(Stop::from(guild_id))?;

    state
        .http
        .create_message(msg.channel_id)
        .content("Stopped the track")?
        .exec()
        .await?;

    Ok(())
}

async fn volume(msg: Message, state: State) -> anyhow::Result<()> {
    tracing::debug!(
        "volume command in channel {} by {}",
        msg.channel_id,
        msg.author.name
    );
    state
        .http
        .create_message(msg.channel_id)
        .content("What's the volume you want to set (0-1000, 100 being the default)?")?
        .exec()
        .await?;

    let author_id = msg.author.id;
    let msg = state
        .standby
        .wait_for_message(msg.channel_id, move |new_msg: &MessageCreate| {
            new_msg.author.id == author_id
        })
        .await?;
    let guild_id = msg.guild_id.unwrap();
    let volume = msg.content.parse::<i64>()?;

    if !(0..=1000).contains(&volume) {
        state
            .http
            .create_message(msg.channel_id)
            .content("That's more than 1000")?
            .exec()
            .await?;

        return Ok(());
    }

    let player = state.lavalink.player(guild_id).await.unwrap();
    player.send(Volume::from((guild_id, volume)))?;

    state
        .http
        .create_message(msg.channel_id)
        .content(&format!("Set the volume to {volume}"))?
        .exec()
        .await?;

    Ok(())
}
