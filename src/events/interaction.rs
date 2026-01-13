use serenity::all::{
    CommandInteraction, ComponentInteraction, Context as SerenityContext, Interaction,
    ModalInteraction,
};
use tracing::error;

use crate::{
    events::EventRouter,
    models::{
        bot::Bot,
        interactions::{
            context::{Context, UnpopulatedContext},
            traits::InteractionHandler,
        },
        response::{ReaperError, ResponseError, ResponseResult},
    },
};

impl EventRouter {
    #[tracing::instrument(skip(self, ctx, interaction))]
    pub async fn on_interaction(&self, ctx: SerenityContext, interaction: Interaction) {
        let result = match interaction {
            Interaction::Command(cmd) => self.handle_command(ctx, &cmd).await,
            Interaction::Component(comp) => self.handle_component(ctx, &comp).await,
            Interaction::Modal(modal) => self.handle_modal(ctx, &modal).await,
            _ => return,
        };

        if let Err(err) = result {
            error!(error = %err, "Error handling interaction");
        }
    }

    #[tracing::instrument(skip(self, ctx, command))]
    async fn handle_command(
        &self,
        ctx: SerenityContext,
        command: &CommandInteraction,
    ) -> ResponseResult<()> {
        let handler = Bot::instance()
            .commands()
            .get(command.data.name.as_str())
            .ok_or(ResponseError::Reaper(ReaperError::CommandNotFound))?;

        let context = self.build_context(ctx, handler.as_ref()).await?;

        handler.execute(context, command).await
    }

    #[tracing::instrument(skip(self, ctx, component))]
    async fn handle_component(
        &self,
        ctx: SerenityContext,
        component: &ComponentInteraction,
    ) -> ResponseResult<()> {
        let handler = Bot::instance()
            .components()
            .get(component.data.custom_id.as_str())
            .ok_or(ResponseError::Reaper(ReaperError::ComponentNotFound))?;

        let context = self.build_context(ctx, handler.as_ref()).await?;

        handler.execute(context, component).await
    }

    #[tracing::instrument(skip(self, ctx, modal))]
    async fn handle_modal(
        &self,
        ctx: SerenityContext,
        modal: &ModalInteraction,
    ) -> ResponseResult<()> {
        let handler = Bot::instance()
            .modals()
            .get(modal.data.custom_id.as_str())
            .ok_or(ResponseError::Reaper(ReaperError::ModalNotFound))?;

        let context = self.build_context(ctx, handler.as_ref()).await?;

        handler.execute(context, modal).await
    }

    #[tracing::instrument(skip(self, ctx, _handler))]
    async fn build_context(
        &self,
        ctx: SerenityContext,
        _handler: &dyn InteractionHandler,
    ) -> ResponseResult<Context> {
        Ok(Context::Unpopulated(UnpopulatedContext {
            serenity_context: ctx,
        }))
    }
}
