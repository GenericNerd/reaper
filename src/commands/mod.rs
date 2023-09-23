use crate::{commands::permissions::PermissionsCommand, models::command::Command};

pub mod permissions;

pub fn get_command_list() -> Vec<Box<dyn Command>> {
    vec![Box::new(PermissionsCommand)]
}
