use serenity::builder::CreateCommand;

pub mod permissions;

pub fn get_command_list() -> Vec<(&'static str, CreateCommand)> {
    vec![("permissions", permissions::register())]
}
