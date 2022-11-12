use serenity::{prelude::{EventHandler, Context}, model::prelude::{interaction::Interaction, Ready}, model::application::command::Command};
use tracing::{error, info};
use crate::{commands, Handler};

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        self.on_command(ctx, interaction).await;
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} has connected!", ready.user.name);
        info!("Beginning command registration");
        let commands = Command::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(|command| commands::permissions::router::register(command))
                .create_application_command(|command| commands::moderation::strike::register(command))
                .create_application_command(|command| commands::moderation::mute::register(command))
                .create_application_command(|command| commands::moderation::kick::register(command))
                .create_application_command(|command| commands::moderation::ban::register(command))
        }).await;
        match commands {
            Ok(commands) => {
                info!("Command registration complete");
                let mut comamnd_names = "Successfully registered commands: ".to_string();
                for command in commands.iter() {
                    comamnd_names.push_str(&command.name);
                    comamnd_names.push_str(", ");
                }
                comamnd_names.pop();
                comamnd_names.pop();
                info!("{}", comamnd_names);
            },
            Err(err) => error!("Command registration failed. The error was: {}", err)
        }
    }
}