use crate::bf::Interpreter;

use std::{error::Error, sync::Arc};

#[allow(clippy::wildcard_imports)]
use super::interaction_prelude::*;
use super::TimInteraction;

#[derive(CommandModel, CreateCommand, Debug)]
#[command(name = "bf", desc = "Compiles and executes Brainfuck code")]
pub struct Command {
    /// The source code to be compiled and executed
    src: String,
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
        match Interpreter::try_from(request.src.clone()) {
            Ok(mut interpreter) => match interpreter.run(8192) {
                Ok(output) => {
                    ia_http
                        .create_response(
                            interaction.id(),
                            interaction.token(),
                            &InteractionResponse {
                                kind: InteractionResponseType::ChannelMessageWithSource,
                                data: Some(
                                    InteractionResponseDataBuilder::new()
                                        .content(format!("Source: ```Brainfuck\n{}\n``` Program output: ```\n{}\n```", request.src, output))
                                        .build()
                                )
                            }
                        )
                        .exec()
                        .await?;
                }
                Err(e) => {
                    ia_http
                        .create_response(
                            interaction.id(),
                            interaction.token(),
                            &InteractionResponse {
                                kind: InteractionResponseType::ChannelMessageWithSource,
                                data: Some(
                                    InteractionResponseDataBuilder::new()
                                        .content(format!(
                                            "Source: ```Brainfuck\n{}\n``` Runtime error: {:?}",
                                            request.src, e
                                        ))
                                        .build(),
                                ),
                            },
                        )
                        .exec()
                        .await?;
                }
            },
            Err(e) => {
                ia_http
                    .create_response(
                        interaction.id(),
                        interaction.token(),
                        &InteractionResponse {
                            kind: InteractionResponseType::ChannelMessageWithSource,
                            data: Some(
                                InteractionResponseDataBuilder::new()
                                    .content(format!(
                                        "Source: ```Brainfuck\n{}\n``` Compile error: {:?}",
                                        request.src, e
                                    ))
                                    .build(),
                            ),
                        },
                    )
                    .exec()
                    .await?;
            }
        }
        Ok(())
    }
}
