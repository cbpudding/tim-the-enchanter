use futures::stream::StreamExt;
use std::{error::Error, fs, sync::Arc};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Event, Intents,
};
use twilight_http::{request::AuditLogReason, Client as HttpClient};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::Interaction,
    gateway::payload::incoming::InteractionCreate,
    guild::Role,
    id::{
        marker::{ApplicationMarker, GuildMarker},
        Id,
    },
};

const TIM: Id<ApplicationMarker> = Id::new(990443765468631081);
const TW: Id<GuildMarker> = Id::new(565884670424645652);

#[derive(CommandModel, CreateCommand, Debug)]
#[command(name = "role", desc = "Assign or unassign a role to a user")]
struct RoleCommand {
    /// The role to be assigned or unassigned
    role: Role,
}

async fn handle_event(
    _shard_id: u64,
    event: Event,
    http: Arc<HttpClient>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let ia_http = http.interaction(TIM);
    match event {
        Event::InteractionCreate(InteractionCreate(interaction)) => {
            let token = interaction.token();
            if let Interaction::ApplicationCommand(command) = interaction.clone() {
                match command.data.name.as_str() {
                    "role" => {
                        let request = RoleCommand::from_interaction(command.data.into())?;
                        if let Some(member) = command.member {
                            if let Some(victim) = member.user {
                                if member.roles.contains(&request.role.id) {
                                    http.remove_guild_member_role(TW, victim.id, request.role.id)
                                        .reason("Unassigned by user")?
                                        .exec()
                                        .await?;
                                    ia_http
                                        .create_followup(token)
                                        .content(
                                            format!("\"{}\" unassigned.", request.role.name)
                                                .as_str(),
                                        )?
                                        .exec()
                                        .await?;
                                } else {
                                    http.add_guild_member_role(TW, victim.id, request.role.id)
                                        .reason("Assigned by user")?
                                        .exec()
                                        .await?;
                                    ia_http
                                        .create_followup(token)
                                        .content(
                                            format!("\"{}\" assigned.", request.role.name).as_str(),
                                        )?
                                        .exec()
                                        .await?;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Read the token from a text file
    let token = fs::read_to_string("token.txt")?;
    // Tell Discord how we're going to create shards(single share only)
    let scheme = ShardScheme::Range {
        from: 0,
        to: 0,
        total: 1,
    };
    // Specify gateway intents
    let intents = Intents::GUILD_MESSAGES | Intents::DIRECT_MESSAGES;
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
    let command_role = RoleCommand::create_command();
    let commands = &[command_role.into()];
    interaction.set_guild_commands(TW, commands).exec().await?;
    // Into the event loop!
    while let Some((shard_id, event)) = events.next().await {
        cache.update(&event);
        tokio::spawn(handle_event(shard_id, event, Arc::clone(&http)));
    }
    Ok(())
}
