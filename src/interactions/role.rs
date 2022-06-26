use std::{error::Error, sync::Arc};

use twilight_http::request::AuditLogReason;
use twilight_model::guild::Role;

#[allow(clippy::wildcard_imports)]
use super::interaction_prelude::*;
use super::TimInteraction;

#[derive(CommandModel, CreateCommand, Debug)]
#[command(name = "role", desc = "Assign or unassign a role to a user")]
pub struct Command {
    /// The role to be assigned or unassigned
    role: Role,
}

#[async_trait]
impl TimInteraction for Command {
    async fn exec(
        interaction: Interaction,
        command: Box<ApplicationCommand>,
        http: Arc<HttpClient>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let ia_http = http.interaction(TIM);
        let request = Self::from_interaction(command.data.into())?;
        if let Some(member) = command.member {
            if let Some(victim) = member.user {
                if member.roles.contains(&request.role.id) {
                    http.remove_guild_member_role(TW, victim.id, request.role.id)
                        .reason("Unassigned by user")?
                        .exec()
                        .await?;
                    ia_http
                        .create_response(
                            interaction.id(),
                            interaction.token(),
                            &InteractionResponse {
                                kind: InteractionResponseType::ChannelMessageWithSource,
                                data: Some(
                                    InteractionResponseDataBuilder::new()
                                        .content(format!("\"{}\" unassigned.", request.role.name))
                                        .build(),
                                ),
                            },
                        )
                        .exec()
                        .await?;
                } else {
                    http.add_guild_member_role(TW, victim.id, request.role.id)
                        .reason("Assigned by user")?
                        .exec()
                        .await?;
                    ia_http
                        .create_response(
                            interaction.id(),
                            interaction.token(),
                            &InteractionResponse {
                                kind: InteractionResponseType::ChannelMessageWithSource,
                                data: Some(
                                    InteractionResponseDataBuilder::new()
                                        .content(format!("\"{}\" assigned.", request.role.name))
                                        .build(),
                                ),
                            },
                        )
                        .exec()
                        .await?;
                }
            }
        }
        Ok(())
    }
}
