#![allow(clippy::unreadable_literal)]
#![feature(box_patterns)]
#![feature(unchecked_math)]

use futures::{stream::StreamExt, TryFutureExt};
use interactions::TimInteraction;
use std::{error::Error, fs, sync::Arc};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Event, Intents,
};
use twilight_http::{request::AuditLogReason, Client as HttpClient};
use twilight_model::{
    application::interaction::Interaction,
    channel::{Reaction, ReactionType},
    gateway::payload::incoming::{InteractionCreate, ReactionAdd},
    id::{
        marker::{ApplicationMarker, GuildMarker},
        Id,
    },
};

const TIM: Id<ApplicationMarker> = Id::new(990443765468631081);
const TW: Id<GuildMarker> = Id::new(565884670424645652);

mod bf;
mod interactions;

#[allow(clippy::single_match)]
async fn handle_interaction(
    interaction: Interaction,
    http: Arc<HttpClient>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Interaction::ApplicationCommand(command) = interaction.clone() {
        match command.data.name.as_str() {
            "bf" => interactions::BfCommand::exec(interaction, command, http).await?,
            "role" => interactions::RoleCommand::exec(interaction, command, http).await?,
            _ => (),
        }
    }
    Ok(())
}

async fn handle_reaction(
    reaction: Reaction,
    http: Arc<HttpClient>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if reaction.message_id == Id::new(991122072883429396) {
        if let ReactionType::Custom { id, .. } = reaction.emoji {
            if id == Id::new(746465402959429672) {
                http.add_guild_member_role(TW, reaction.user_id, Id::new(991122401284870224))
                    .reason("Passed server verification")?
                    .exec()
                    .await?;
            }
        }
    }
    Ok(())
}

#[allow(clippy::single_match)]
async fn handle_event(
    _shard_id: u64,
    event: Event,
    http: Arc<HttpClient>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        Event::InteractionCreate(InteractionCreate(interaction)) => {
            handle_interaction(interaction, http).await
        }
        Event::ReactionAdd(box ReactionAdd(reaction)) => handle_reaction(reaction, http).await,
        _ => Ok(()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Read the token from a text file
    let token = fs::read_to_string("token.txt")?.trim().to_string();
    // Tell Discord how we're going to create shards(single share only)
    let scheme = ShardScheme::Range {
        from: 0,
        to: 0,
        total: 1,
    };
    // Specify gateway intents
    let intents =
        Intents::GUILD_MESSAGES | Intents::DIRECT_MESSAGES | Intents::GUILD_MESSAGE_REACTIONS;
    // Build and start the cluster
    let (cluster, mut events) = Cluster::builder(token.clone(), intents)
        .shard_scheme(scheme)
        .build()
        .await?;
    let cluster = Arc::new(cluster);
    let cluster_spawn = cluster.clone();
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });
    // Initialize the HTTP client
    let http = Arc::new(HttpClient::new(token));
    let interaction = http.interaction(TIM);
    // Initialize the cache
    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE)
        .build();
    // Register commands with Discord
    let commands = interactions::get_commands().map(std::convert::Into::into);
    interaction.set_guild_commands(TW, &commands).exec().await?;
    // Into the event loop!
    while let Some((shard_id, event)) = events.next().await {
        cache.update(&event);
        tokio::spawn(
            handle_event(shard_id, event, Arc::clone(&http))
                .map_err(|err| println!("Error processing event: {:?}", err)),
        );
    }
    Ok(())
}
