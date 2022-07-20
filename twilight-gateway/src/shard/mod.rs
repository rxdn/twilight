//! Types for working with and running connections to the gateway.
//!
//! At the heart of the `shard` module is the [`Shard`] itself: it's the
//! interface used to start a shard, send messages to the gateway, and receive
//! [`Event`]s from it, such as [new messages] or [channel deletions].
//!
//! Once running, the shard maintains [information about itself] that you can
//! obtain through it. This is information such as the latency or the current
//! [`Stage`] of the connection, like whether it's [`Disconnected`] or
//! [`Resuming`] the connection.
//!
//! Shards are configurable through the [`ShardBuilder`], which provides a clean
//! interface for correctly configuring a shard.
//!
//! # Member Chunking
//!
//! Requesting chunks of a guild's members may be done via [`Shard::command`]
//! and [`RequestGuildMembers`]. For example, requesting chunks of members whose
//! names start with "tw":
//!
//! ```no_run
//! # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use std::env;
//! use twilight_gateway::{shard::Shard, Intents};
//! use twilight_model::{
//!     gateway::payload::outgoing::RequestGuildMembers,
//!     id::Id,
//! };
//!
//! let intents = Intents::GUILD_MEMBERS;
//! let token = env::var("DISCORD_TOKEN")?;
//!
//! let (shard, _events) = Shard::new(token, intents);
//! shard.start().await?;
//!
//! // Query members whose names start with "tw" and limit the results to
//! // 10 members.
//! let request =
//!     RequestGuildMembers::builder(Id::new(1))
//!         .query("tw", Some(10));
//!
//! // Send the request over the shard.
//! shard.command(&request).await?;
//! # Ok(()) }
//! ```
//!
//! [`Disconnected`]: Stage::Disconnected
//! [`Event`]: ::twilight_model::gateway::event::Event
//! [`RequestGuildMembers`]: twilight_model::gateway::payload::outgoing::RequestGuildMembers
//! [`Resuming`]: Stage::Resuming
//! [channel deletions]: ::twilight_model::gateway::event::Event::ChannelDelete
//! [information about itself]: Shard::info
//! [new messages]: ::twilight_model::gateway::event::Event::MessageCreate

pub mod raw_message;
pub mod stage;

mod builder;
mod command;
mod config;
mod emitter;
mod event;
mod r#impl;
mod json;
mod processor;
#[cfg(any(
    feature = "native",
    feature = "rustls-native-roots",
    feature = "rustls-webpki-roots"
))]
pub(crate) mod tls;

pub use self::{
    builder::{ShardBuilder, ShardIdError, ShardIdErrorType},
    command::Command,
    config::Config,
    event::Events,
    processor::heartbeat::Latency,
    r#impl::{
        CommandError, CommandErrorType, Information, ResumeSession, SendError, SendErrorType,
        SessionInactiveError, Shard, ShardStartError, ShardStartErrorType,
    },
    stage::Stage,
};

use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

type ShardStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
