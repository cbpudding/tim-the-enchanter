use std::{error::Error, sync::Arc};

use async_trait::async_trait;
use twilight_http::Client as HttpClient;
use twilight_interactions::command::{ApplicationCommandData, CreateCommand};
use twilight_model::application::interaction::{ApplicationCommand, Interaction};

mod bf;
mod role;

pub use bf::Command as BfCommand;
pub use role::Command as RoleCommand;

pub fn get_commands() -> [ApplicationCommandData; 2] {
    [
        BfCommand::create_command(),
        RoleCommand::create_command()
    ]
}

mod interaction_prelude {
    pub use async_trait::async_trait;
    pub use twilight_http::Client as HttpClient;
    pub use twilight_interactions::command::{CommandModel, CreateCommand};
    pub use twilight_model::{
        application::interaction::{ApplicationCommand, Interaction},
        http::interaction::{InteractionResponse, InteractionResponseType},
    };
    pub use twilight_util::builder::InteractionResponseDataBuilder;

    pub(super) use crate::{TIM, TW};
}

#[async_trait]
pub trait TimInteraction {
    async fn exec(
        interaction: Interaction,
        command: Box<ApplicationCommand>,
        http: Arc<HttpClient>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}
