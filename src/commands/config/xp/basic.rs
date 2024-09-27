use serenity::all::{
    ActionRowComponent, ButtonStyle, CommandInteraction, CreateActionRow, CreateButton,
    CreateEmbed, CreateInputText, CreateInteractionResponse, CreateModal, InputTextStyle,
};

use crate::{
    commands::config::{ConfigError, ConfigStage, EMBED_COLOR},
    models::{
        command::{CommandContext, CommandContextReply},
        handler::Handler,
        response::{Response, ResponseError},
    },
};

use super::XP_TITLE;

pub struct XPEnter;
#[async_trait::async_trait]
impl ConfigStage for XPEnter {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        let message = ctx
            .reply_get_message(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(XP_TITLE)
                            .description("Would you like to configure levelling?")
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new("yes")
                            .label("Yes")
                            .style(ButtonStyle::Success),
                        CreateButton::new("no")
                            .label("No")
                            .style(ButtonStyle::Secondary),
                    ])]),
            )
            .await?;

        let collector = message
            .await_component_interaction(&ctx.ctx)
            .author_id(cmd.user.id)
            .timeout(std::time::Duration::new(60, 0));

        if let Some(interaction) = collector.await {
            interaction
                .create_response(
                    &ctx.ctx.http,
                    serenity::builder::CreateInteractionResponse::Acknowledge,
                )
                .await?;
            match interaction.data.custom_id.as_str() {
                "yes" => {
                    if sqlx::query!(
                        "SELECT guild_id FROM xp_configuration WHERE guild_id = $1",
                        ctx.guild.id.get() as i64
                    )
                    .fetch_optional(&handler.main_database)
                    .await?
                    .is_none()
                    {
                        sqlx::query!(
                            "INSERT INTO xp_configuration (guild_id) VALUES ($1)",
                            ctx.guild.id.get() as i64
                        )
                        .execute(&handler.main_database)
                        .await?;
                    }
                    return Ok(None);
                }
                "no" => {
                    return Ok(Some(22));
                }
                _ => {
                    return Err(ConfigError {
                        error: ResponseError::Execution(
                            "Invalid option",
                            Some("Please select a valid option.".to_string()),
                        ),
                        stages_to_skip: None,
                    })
                }
            }
        }
        Err(ConfigError {
            error: ResponseError::Execution(
                "Time out",
                Some("We didn't get a response in time. Please try again.".to_string()),
            ),
            stages_to_skip: Some(100),
        })
    }
}

pub struct XPSetOrRandom;
#[async_trait::async_trait]
impl ConfigStage for XPSetOrRandom {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        let Some(xp_configuration) = sqlx::query!(
            "SELECT min_xp_per_message, max_xp_per_message, set_xp_per_message FROM xp_configuration WHERE guild_id = $1",
            ctx.guild.id.get() as i64
        )
        .fetch_optional(&handler.main_database)
        .await? else {
            sqlx::query!(
                "INSERT INTO xp_configuration (guild_id) VALUES ($1)",
                ctx.guild.id.get() as i64
            )
            .execute(&handler.main_database)
            .await?;
            return Ok(Some(0));
        };

        let current_setting_string = match xp_configuration.set_xp_per_message {
            Some(set_xp) => format!("**{set_xp}** XP"),
            None => format!(
                "**{}** to **{}** XP",
                xp_configuration.min_xp_per_message.unwrap(),
                xp_configuration.max_xp_per_message.unwrap()
            ),
        };

        let message = ctx
            .reply_get_message(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(XP_TITLE)
                            .description(format!(
                                "When earning XP, would you like users to earn a specific amount of XP or a random amount?\n\n{}\n\nYour current setting is: {current_setting_string}",
                                "If you choose a random amount, you will be asked to set the minimum and maximum XP that can be earned per message.",
                            ))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new("random")
                            .label("Random")
                            .style(ButtonStyle::Primary),
                        CreateButton::new("specific")
                            .label("Specific")
                            .style(ButtonStyle::Primary),
                        CreateButton::new("skip")
                            .label("Skip")
                            .style(ButtonStyle::Secondary)
                    ])]),
            )
            .await?;

        let collector = message
            .await_component_interaction(&ctx.ctx)
            .author_id(cmd.user.id)
            .timeout(std::time::Duration::new(60, 0));

        if let Some(interaction) = collector.await {
            match interaction.data.custom_id.as_str() {
                "random" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            CreateInteractionResponse::Modal(
                                CreateModal::new("random_modal", "Random XP Configuration")
                                    .components(vec![
                                        CreateActionRow::InputText(
                                            CreateInputText::new(
                                                InputTextStyle::Short,
                                                "Minimum XP",
                                                "min_xp",
                                            )
                                            .placeholder("Recommended: 15")
                                            .required(true),
                                        ),
                                        CreateActionRow::InputText(
                                            CreateInputText::new(
                                                InputTextStyle::Short,
                                                "Maximum XP",
                                                "max_xp",
                                            )
                                            .placeholder("Recommended: 40")
                                            .required(true),
                                        ),
                                    ]),
                            ),
                        )
                        .await?;

                    let modal_collector = message
                        .await_modal_interaction(&ctx.ctx)
                        .author_id(cmd.user.id)
                        .timeout(std::time::Duration::new(60, 0));

                    if let Some(interaction) = modal_collector.await {
                        interaction
                            .create_response(&ctx.ctx.http, CreateInteractionResponse::Acknowledge)
                            .await?;

                        if let ActionRowComponent::InputText(text) =
                            &interaction.data.components[0].components[0]
                        {
                            let Ok(min_xp) = text.value.as_ref().unwrap().parse::<i32>() else {
                                return Err(ConfigError {
                                    error: ResponseError::Execution(
                                        "Invalid minimum XP",
                                        Some("Please enter a valid minimum XP.".to_string()),
                                    ),
                                    stages_to_skip: None,
                                });
                            };
                            if min_xp < 0 {
                                return Err(ConfigError {
                                    error: ResponseError::Execution(
                                        "Invalid minimum XP",
                                        Some("Please enter a valid minimum XP.".to_string()),
                                    ),
                                    stages_to_skip: None,
                                });
                            }

                            sqlx::query!(
                                "UPDATE xp_configuration SET min_xp_per_message = $2, set_xp_per_message = NULL WHERE guild_id=$1",
                                ctx.guild.id.get() as i64,
                                min_xp
                            )
                            .execute(&handler.main_database)
                            .await?;
                        }

                        if let ActionRowComponent::InputText(text) =
                            &interaction.data.components[1].components[0]
                        {
                            let Ok(max_xp) = text.value.as_ref().unwrap().parse::<i32>() else {
                                return Err(ConfigError {
                                    error: ResponseError::Execution(
                                        "Invalid minimum XP",
                                        Some("Please enter a valid minimum XP.".to_string()),
                                    ),
                                    stages_to_skip: None,
                                });
                            };
                            if max_xp < 0 {
                                return Err(ConfigError {
                                    error: ResponseError::Execution(
                                        "Invalid minimum XP",
                                        Some("Please enter a valid minimum XP.".to_string()),
                                    ),
                                    stages_to_skip: None,
                                });
                            }

                            sqlx::query!(
                                "UPDATE xp_configuration SET max_xp_per_message = $2 WHERE guild_id=$1",
                                ctx.guild.id.get() as i64,
                                max_xp
                            )
                            .execute(&handler.main_database)
                            .await?;
                        }

                        return Ok(None);
                    }
                }
                "specific" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            CreateInteractionResponse::Modal(
                                CreateModal::new("set_modal", "Set XP Configuration").components(
                                    vec![CreateActionRow::InputText(
                                        CreateInputText::new(
                                            InputTextStyle::Short,
                                            "Set XP",
                                            "set_xp",
                                        )
                                        .placeholder("Recommended: 25")
                                        .required(true),
                                    )],
                                ),
                            ),
                        )
                        .await?;

                    let modal_collector = message
                        .await_modal_interaction(&ctx.ctx)
                        .author_id(cmd.user.id)
                        .timeout(std::time::Duration::new(60, 0));

                    if let Some(interaction) = modal_collector.await {
                        interaction
                            .create_response(&ctx.ctx.http, CreateInteractionResponse::Acknowledge)
                            .await?;

                        if let ActionRowComponent::InputText(text) =
                            &interaction.data.components[0].components[0]
                        {
                            let Ok(set_xp) = text.value.as_ref().unwrap().parse::<i32>() else {
                                return Err(ConfigError {
                                    error: ResponseError::Execution(
                                        "Invalid XP",
                                        Some("Please enter a valid XP.".to_string()),
                                    ),
                                    stages_to_skip: None,
                                });
                            };
                            if set_xp < 0 {
                                return Err(ConfigError {
                                    error: ResponseError::Execution(
                                        "Invalid XP",
                                        Some("Please enter a valid XP.".to_string()),
                                    ),
                                    stages_to_skip: None,
                                });
                            }

                            sqlx::query!(
                                "UPDATE xp_configuration SET set_xp_per_message = $2, min_xp_per_message = NULL, max_xp_per_message = NULL WHERE guild_id=$1",
                                ctx.guild.id.get() as i64,
                                set_xp
                            )
                            .execute(&handler.main_database)
                            .await?;
                        }
                        return Ok(None);
                    }
                }
                "skip" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            serenity::builder::CreateInteractionResponse::Acknowledge,
                        )
                        .await?;

                    return Ok(None);
                }
                _ => {
                    return Err(ConfigError {
                        error: ResponseError::Execution(
                            "Invalid option",
                            Some("Please select a valid option.".to_string()),
                        ),
                        stages_to_skip: None,
                    })
                }
            }
        }
        Err(ConfigError {
            error: ResponseError::Execution(
                "Time out",
                Some("We didn't get a response in time. Please try again.".to_string()),
            ),
            stages_to_skip: Some(100),
        })
    }
}

pub struct XPMessageCooldown;
#[async_trait::async_trait]
impl ConfigStage for XPMessageCooldown {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        let cooldown = sqlx::query!(
            "SELECT message_cooldown FROM xp_configuration WHERE guild_id = $1",
            ctx.guild.id.get() as i64
        )
        .fetch_one(&handler.main_database)
        .await?
        .message_cooldown;

        let message = ctx
            .reply_get_message(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(XP_TITLE)
                            .description(format!("How much time should pass after sending a message before you can earn XP again?\n\nThe current cooldown is: **{cooldown}**s"))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new("change")
                            .label("Change")
                            .style(ButtonStyle::Success),
                        CreateButton::new("skip")
                            .label("Skip")
                            .style(ButtonStyle::Secondary)
                    ])]),
            )
            .await?;

        let collector = message
            .await_component_interaction(&ctx.ctx)
            .author_id(cmd.user.id)
            .timeout(std::time::Duration::new(60, 0));

        if let Some(interaction) = collector.await {
            match interaction.data.custom_id.as_str() {
                "skip" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            serenity::builder::CreateInteractionResponse::Acknowledge,
                        )
                        .await?;
                    return Ok(None);
                }
                "change" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            CreateInteractionResponse::Modal(
                                CreateModal::new("message_cooldown_modal", "Message Cooldown")
                                    .components(vec![CreateActionRow::InputText(
                                        CreateInputText::new(
                                            InputTextStyle::Short,
                                            "Cooldown in seconds",
                                            "message_cooldown",
                                        )
                                        .placeholder("Recommended: 60")
                                        .required(true),
                                    )]),
                            ),
                        )
                        .await?;

                    let modal_collector = message
                        .await_modal_interaction(&ctx.ctx)
                        .author_id(cmd.user.id)
                        .timeout(std::time::Duration::new(60, 0));

                    if let Some(interaction) = modal_collector.await {
                        interaction
                            .create_response(
                                &ctx.ctx.http,
                                serenity::builder::CreateInteractionResponse::Acknowledge,
                            )
                            .await?;

                        if let ActionRowComponent::InputText(text) =
                            &interaction.data.components[0].components[0]
                        {
                            let Ok(cooldown) = text.value.as_ref().unwrap().parse::<i32>() else {
                                return Err(ConfigError {
                                    error: ResponseError::Execution(
                                        "Invalid cooldown",
                                        Some("Please enter a valid cooldown.".to_string()),
                                    ),
                                    stages_to_skip: None,
                                });
                            };
                            if cooldown < 1 {
                                return Err(ConfigError {
                                    error: ResponseError::Execution(
                                        "Invalid cooldown",
                                        Some("Please enter a valid cooldown.".to_string()),
                                    ),
                                    stages_to_skip: None,
                                });
                            }

                            sqlx::query!(
                                "UPDATE xp_configuration SET message_cooldown = $2 WHERE guild_id=$1",
                                ctx.guild.id.get() as i64,
                                cooldown
                            )
                            .execute(&handler.main_database)
                            .await?;
                        }
                        return Ok(None);
                    }
                }
                _ => {
                    return Err(ConfigError {
                        error: ResponseError::Execution(
                            "Invalid option",
                            Some("Please select a valid option.".to_string()),
                        ),
                        stages_to_skip: None,
                    })
                }
            }
        }
        Err(ConfigError {
            error: ResponseError::Execution(
                "Time out",
                Some("We didn't get a response in time. Please try again.".to_string()),
            ),
            stages_to_skip: Some(100),
        })
    }
}

pub struct XPMaxLevelEnable;
#[async_trait::async_trait]
impl ConfigStage for XPMaxLevelEnable {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        let max_level = sqlx::query!(
            "SELECT max_level FROM xp_configuration WHERE guild_id = $1",
            ctx.guild.id.get() as i64
        )
        .fetch_one(&handler.main_database)
        .await?
        .max_level;
        let max_level_string = match max_level {
            Some(max_level) => {
                format!("Level **{max_level}**")
            }
            None => "**No maximum**".to_string(),
        };

        let message = ctx
            .reply_get_message(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(XP_TITLE)
                            .description(format!("What should be the maximum level a user can reach?\n\nYour current setting is: {max_level_string}"))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new("enable")
                            .label("Set a maximum")
                            .style(ButtonStyle::Success),
                        CreateButton::new("disable")
                            .label("No maximum")
                            .style(ButtonStyle::Danger),
                        CreateButton::new("skip")
                            .label("Skip")
                            .style(ButtonStyle::Secondary),
                    ])]),
            )
            .await?;

        let collector = message
            .await_component_interaction(&ctx.ctx)
            .author_id(cmd.user.id)
            .timeout(std::time::Duration::new(60, 0));

        if let Some(interaction) = collector.await {
            match interaction.data.custom_id.as_str() {
                "skip" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            serenity::builder::CreateInteractionResponse::Acknowledge,
                        )
                        .await?;
                    return Ok(None);
                }
                "enable" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            CreateInteractionResponse::Modal(
                                CreateModal::new("max_level_modal", "Max Level").components(vec![
                                    CreateActionRow::InputText(
                                        CreateInputText::new(
                                            InputTextStyle::Short,
                                            "Max Level",
                                            "max_level",
                                        )
                                        .placeholder("Recommended: 100")
                                        .required(true),
                                    ),
                                ]),
                            ),
                        )
                        .await?;

                    let modal_collector = message
                        .await_modal_interaction(&ctx.ctx)
                        .author_id(cmd.user.id)
                        .timeout(std::time::Duration::new(60, 0));

                    if let Some(interaction) = modal_collector.await {
                        interaction
                            .create_response(
                                &ctx.ctx.http,
                                serenity::builder::CreateInteractionResponse::Acknowledge,
                            )
                            .await?;

                        if let ActionRowComponent::InputText(text) =
                            &interaction.data.components[0].components[0]
                        {
                            let Ok(max_level) = text.value.as_ref().unwrap().parse::<i32>() else {
                                return Err(ConfigError {
                                    error: ResponseError::Execution(
                                        "Invalid max level",
                                        Some("Please enter a valid max level.".to_string()),
                                    ),
                                    stages_to_skip: None,
                                });
                            };
                            if max_level < 1 {
                                return Err(ConfigError {
                                    error: ResponseError::Execution(
                                        "Invalid max level",
                                        Some("Please enter a valid max level.".to_string()),
                                    ),
                                    stages_to_skip: None,
                                });
                            }

                            sqlx::query!(
                                "UPDATE xp_configuration SET max_level = $2 WHERE guild_id=$1",
                                ctx.guild.id.get() as i64,
                                max_level
                            )
                            .execute(&handler.main_database)
                            .await?;
                        }
                        return Ok(None);
                    }
                }
                "disable" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            serenity::builder::CreateInteractionResponse::Acknowledge,
                        )
                        .await?;

                    sqlx::query!(
                        "UPDATE xp_configuration SET max_level = NULL WHERE guild_id=$1",
                        ctx.guild.id.get() as i64
                    )
                    .execute(&handler.main_database)
                    .await?;
                    return Ok(None);
                }
                _ => {
                    return Err(ConfigError {
                        error: ResponseError::Execution(
                            "Invalid option",
                            Some("Please select a valid option.".to_string()),
                        ),
                        stages_to_skip: None,
                    })
                }
            }
        }
        Err(ConfigError {
            error: ResponseError::Execution(
                "Time out",
                Some("We didn't get a response in time. Please try again.".to_string()),
            ),
            stages_to_skip: Some(100),
        })
    }
}

pub struct XPStackRewards;
#[async_trait::async_trait]
impl ConfigStage for XPStackRewards {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        let stack_rewards = sqlx::query!(
            "SELECT stack_rewards FROM xp_configuration WHERE guild_id = $1",
            ctx.guild.id.get() as i64
        )
        .fetch_one(&handler.main_database)
        .await?
        .stack_rewards;
        let stack_rewards_string = if stack_rewards {
            "**Enabled**"
        } else {
            "**Disabled**"
        };

        let help_text = r"> Enabling this allows each user to keep all of their roles earned by levelling up. By default, users will have their current role replaced by their next role reward.";

        let message = ctx
            .reply_get_message(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(XP_TITLE)
                            .description(format!("Would you like to enable stacking rewards?\n\n{help_text}\n\nYour current setting is: {stack_rewards_string}"))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new("enable")
                            .label("Enable")
                            .style(ButtonStyle::Success),
                        CreateButton::new("disable")
                            .label("Disable")
                            .style(ButtonStyle::Danger),
                        CreateButton::new("skip")
                            .label("Skip")
                            .style(ButtonStyle::Secondary),
                    ])]),
            )
            .await?;

        let collector = message
            .await_component_interaction(&ctx.ctx)
            .author_id(cmd.user.id)
            .timeout(std::time::Duration::new(60, 0));

        if let Some(interaction) = collector.await {
            match interaction.data.custom_id.as_str() {
                "skip" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            serenity::builder::CreateInteractionResponse::Acknowledge,
                        )
                        .await?;
                    return Ok(None);
                }
                "enable" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            serenity::builder::CreateInteractionResponse::Acknowledge,
                        )
                        .await?;

                    sqlx::query!(
                        "UPDATE xp_configuration SET stack_rewards = true WHERE guild_id = $1",
                        ctx.guild.id.get() as i64
                    )
                    .execute(&handler.main_database)
                    .await?;
                    return Ok(None);
                }
                "disable" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            serenity::builder::CreateInteractionResponse::Acknowledge,
                        )
                        .await?;

                    sqlx::query!(
                        "UPDATE xp_configuration SET stack_rewards = false WHERE guild_id = $1",
                        ctx.guild.id.get() as i64
                    )
                    .execute(&handler.main_database)
                    .await?;
                    return Ok(None);
                }
                _ => {
                    return Err(ConfigError {
                        error: ResponseError::Execution(
                            "Invalid option",
                            Some("Please select a valid option.".to_string()),
                        ),
                        stages_to_skip: None,
                    })
                }
            }
        }
        Err(ConfigError {
            error: ResponseError::Execution(
                "Time out",
                Some("We didn't get a response in time. Please try again.".to_string()),
            ),
            stages_to_skip: Some(100),
        })
    }
}

pub struct XPStackMultipliers;
#[async_trait::async_trait]
impl ConfigStage for XPStackMultipliers {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        let stack_multipliers = sqlx::query!(
            "SELECT stack_multipliers FROM xp_configuration WHERE guild_id = $1",
            ctx.guild.id.get() as i64
        )
        .fetch_one(&handler.main_database)
        .await?
        .stack_multipliers;
        let stack_multipliers_string = if stack_multipliers {
            "**Enabled**"
        } else {
            "**Disabled**"
        };

        let help_text = r"> By default, users will have each of their boosts stacked.
> 
> For example, a user with a role that gives a *20% boost* chatting in a channel with a *10% boost* will receive a total *30% boost* to their XP.
> 
> Disabling this will only give the user their highest boost, which in the above case would be a *20% boost*.";

        let message = ctx
            .reply_get_message(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(XP_TITLE)
                            .description(format!("{}?\n\n{help_text}\n\nYour current setting is: {stack_multipliers_string}",
                            if stack_multipliers {
                                "Would you like to configure XP boosts?"
                            } else {
                                "Would you like to enable stacking XP boosts?"
                            }))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new("enable")
                            .label(if stack_multipliers {
                                "Configure"
                            } else {
                                "Enable"
                            })
                            .style(ButtonStyle::Success),
                        CreateButton::new("disable")
                            .label("Disable")
                            .style(ButtonStyle::Danger),
                        CreateButton::new("skip")
                            .label("Skip")
                            .style(ButtonStyle::Secondary),
                    ])]),
            )
            .await?;

        let collector = message
            .await_component_interaction(&ctx.ctx)
            .author_id(cmd.user.id)
            .timeout(std::time::Duration::new(60, 0));

        if let Some(interaction) = collector.await {
            match interaction.data.custom_id.as_str() {
                "skip" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            serenity::builder::CreateInteractionResponse::Acknowledge,
                        )
                        .await?;
                    return Ok(Some(2));
                }
                "enable" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            serenity::builder::CreateInteractionResponse::Acknowledge,
                        )
                        .await?;

                    sqlx::query!(
                        "UPDATE xp_configuration SET stack_multipliers = true WHERE guild_id = $1",
                        ctx.guild.id.get() as i64
                    )
                    .execute(&handler.main_database)
                    .await?;
                    return Ok(None);
                }
                "disable" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            serenity::builder::CreateInteractionResponse::Acknowledge,
                        )
                        .await?;

                    sqlx::query!(
                        "UPDATE xp_configuration SET stack_multipliers = false, multiplier_cap = NULL WHERE guild_id = $1",
                        ctx.guild.id.get() as i64
                    )
                    .execute(&handler.main_database)
                    .await?;
                    return Ok(Some(2));
                }
                _ => {
                    return Err(ConfigError {
                        error: ResponseError::Execution(
                            "Invalid option",
                            Some("Please select a valid option.".to_string()),
                        ),
                        stages_to_skip: None,
                    })
                }
            }
        }
        Err(ConfigError {
            error: ResponseError::Execution(
                "Time out",
                Some("We didn't get a response in time. Please try again.".to_string()),
            ),
            stages_to_skip: Some(100),
        })
    }
}

pub struct XPMultiplierCap;
#[async_trait::async_trait]
impl ConfigStage for XPMultiplierCap {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        let multiplier_cap = sqlx::query!(
            "SELECT multiplier_cap FROM xp_configuration WHERE guild_id = $1",
            ctx.guild.id.get() as i64
        )
        .fetch_one(&handler.main_database)
        .await?
        .multiplier_cap;
        let multiplier_cap_string = match multiplier_cap {
            Some(multiplier_cap) => {
                format!(
                    "**A maximum boost of {}%**",
                    ((multiplier_cap - 1.0) * 100.0).round() as i32
                )
            }
            None => "**No Maximum**".to_string(),
        };

        let help_text = r"> Setting an XP boost cap limits the percentage increase a user can obtain.
> 
> For example, a user with a role that gives a 20% boost chatting in a channel with a 10% boost will receive a total 30% boost to their XP.
> 
> If you're setting an XP boost cap of 25% the above user would only receive a 25% boost.";

        let message = ctx
            .reply_get_message(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(XP_TITLE)
                            .description(format!("What should be the maximum XP boost users can obtain?\n\n{help_text}\n\nYour current setting is: {multiplier_cap_string}"))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new("enable")
                            .label("Set a maximum")
                            .style(ButtonStyle::Success),
                        CreateButton::new("disable")
                            .label("No maximum")
                            .style(ButtonStyle::Danger),
                        CreateButton::new("skip")
                            .label("Skip")
                            .style(ButtonStyle::Secondary),
                    ])]),
            )
            .await?;

        let collector = message
            .await_component_interaction(&ctx.ctx)
            .author_id(cmd.user.id)
            .timeout(std::time::Duration::new(60, 0));

        if let Some(interaction) = collector.await {
            match interaction.data.custom_id.as_str() {
                "skip" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            serenity::builder::CreateInteractionResponse::Acknowledge,
                        )
                        .await?;
                    return Ok(None);
                }
                "enable" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            CreateInteractionResponse::Modal(
                                CreateModal::new("multiplier_cap_modal", "Multiplier Cap")
                                    .components(vec![CreateActionRow::InputText(
                                        CreateInputText::new(
                                            InputTextStyle::Short,
                                            "Multiplier Cap in %",
                                            "multiplier_cap",
                                        )
                                        .placeholder("Recommended: 50")
                                        .required(true),
                                    )]),
                            ),
                        )
                        .await?;

                    let modal_collector = message
                        .await_modal_interaction(&ctx.ctx)
                        .author_id(cmd.user.id)
                        .timeout(std::time::Duration::new(60, 0));

                    if let Some(interaction) = modal_collector.await {
                        interaction
                            .create_response(
                                &ctx.ctx.http,
                                serenity::builder::CreateInteractionResponse::Acknowledge,
                            )
                            .await?;

                        if let ActionRowComponent::InputText(text) =
                            &interaction.data.components[0].components[0]
                        {
                            let Ok(multiplier_cap) = text.value.as_ref().unwrap().parse::<i32>()
                            else {
                                return Err(ConfigError {
                                    error: ResponseError::Execution(
                                        "Invalid multiplier cap",
                                        Some("Please enter a valid multiplier cap.".to_string()),
                                    ),
                                    stages_to_skip: None,
                                });
                            };
                            if multiplier_cap < 0 {
                                return Err(ConfigError {
                                    error: ResponseError::Execution(
                                        "Invalid multiplier cap",
                                        Some("Please enter a valid multiplier cap.".to_string()),
                                    ),
                                    stages_to_skip: None,
                                });
                            }

                            sqlx::query!(
                                "UPDATE xp_configuration SET multiplier_cap = $2 WHERE guild_id=$1",
                                ctx.guild.id.get() as i64,
                                (multiplier_cap as f32 / 100.0) + 1.0
                            )
                            .execute(&handler.main_database)
                            .await?;
                        }
                        return Ok(None);
                    }
                }
                "disable" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            serenity::builder::CreateInteractionResponse::Acknowledge,
                        )
                        .await?;

                    sqlx::query!(
                        "UPDATE xp_configuration SET multiplier_cap = NULL WHERE guild_id = $1",
                        ctx.guild.id.get() as i64
                    )
                    .execute(&handler.main_database)
                    .await?;
                    return Ok(Some(2));
                }
                _ => {
                    return Err(ConfigError {
                        error: ResponseError::Execution(
                            "Invalid option",
                            Some("Please select a valid option.".to_string()),
                        ),
                        stages_to_skip: None,
                    })
                }
            }
        }
        Err(ConfigError {
            error: ResponseError::Execution(
                "Time out",
                Some("We didn't get a response in time. Please try again.".to_string()),
            ),
            stages_to_skip: Some(100),
        })
    }
}

pub struct XPResetLevelOnLeave;
#[async_trait::async_trait]
impl ConfigStage for XPResetLevelOnLeave {
    async fn execute(
        &self,
        handler: &Handler,
        ctx: &CommandContext,
        cmd: &CommandInteraction,
    ) -> Result<Option<usize>, ConfigError> {
        let reset_level_on_leave = sqlx::query!(
            "SELECT reset_level_on_leave FROM xp_configuration WHERE guild_id = $1",
            ctx.guild.id.get() as i64
        )
        .fetch_one(&handler.main_database)
        .await?
        .reset_level_on_leave;
        let reset_level_on_leave_string = if reset_level_on_leave {
            "**Enabled**"
        } else {
            "**Disabled**"
        };

        let message = ctx
            .reply_get_message(
                cmd,
                Response::new()
                    .embed(
                        CreateEmbed::new()
                            .title(XP_TITLE)
                            .description(format!(
                                "Would you like to reset a user's level when they leave the server?\n\nYour current setting is: {reset_level_on_leave_string}"
                            ))
                            .color(EMBED_COLOR),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new("enable")
                            .label("Enable")
                            .style(ButtonStyle::Success),
                        CreateButton::new("disable")
                            .label("Disable")
                            .style(ButtonStyle::Danger),
                        CreateButton::new("skip")
                            .label("Skip")
                            .style(ButtonStyle::Secondary),
                    ])]),
            )
            .await?;

        let collector = message
            .await_component_interaction(&ctx.ctx)
            .author_id(cmd.user.id)
            .timeout(std::time::Duration::new(60, 0));

        if let Some(interaction) = collector.await {
            match interaction.data.custom_id.as_str() {
                "skip" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            serenity::builder::CreateInteractionResponse::Acknowledge,
                        )
                        .await?;
                    return Ok(None);
                }
                "enable" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            serenity::builder::CreateInteractionResponse::Acknowledge,
                        )
                        .await?;

                    sqlx::query!(
                        "UPDATE xp_configuration SET reset_level_on_leave = true WHERE guild_id = $1",
                        ctx.guild.id.get() as i64
                    )
                    .execute(&handler.main_database)
                    .await?;
                    return Ok(None);
                }
                "disable" => {
                    interaction
                        .create_response(
                            &ctx.ctx.http,
                            serenity::builder::CreateInteractionResponse::Acknowledge,
                        )
                        .await?;

                    sqlx::query!(
                        "UPDATE xp_configuration SET reset_level_on_leave = false WHERE guild_id = $1",
                        ctx.guild.id.get() as i64
                    )
                    .execute(&handler.main_database)
                    .await?;
                    return Ok(None);
                }
                _ => {
                    return Err(ConfigError {
                        error: ResponseError::Execution(
                            "Invalid option",
                            Some("Please select a valid option.".to_string()),
                        ),
                        stages_to_skip: None,
                    })
                }
            }
        }
        Err(ConfigError {
            error: ResponseError::Execution(
                "Time out",
                Some("We didn't get a response in time. Please try again.".to_string()),
            ),
            stages_to_skip: Some(100),
        })
    }
}
