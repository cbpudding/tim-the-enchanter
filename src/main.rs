use futures::stream::StreamExt;
use std::{error::Error, fs, sync::Arc};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Event, Intents,
};
use twilight_http::Client as HttpClient;

async fn handle_event(
    _shard_id: u64,
    event: Event,
    _http: Arc<HttpClient>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
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
    // Initialize the cache
    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE)
        .build();
    // Into the event loop!
    while let Some((shard_id, event)) = events.next().await {
        cache.update(&event);
        tokio::spawn(handle_event(shard_id, event, Arc::clone(&http)));
    }
    Ok(())
}
