use serenity::all::{CommandDataOption, CommandDataOptionValue, CommandInteraction};

use crate::models::{role::Role, user::User};

pub struct Options {
    options: Vec<CommandDataOption>,
}

impl Options {
    pub fn new(options: Vec<CommandDataOption>) -> Self {
        Self { options }
    }

    pub fn from_command(cmd: &CommandInteraction) -> Self {
        Self {
            options: cmd.data.options.clone(),
        }
    }

    pub fn get_user(&self, name: &str) -> Option<User> {
        for option in &self.options {
            match &option.value {
                CommandDataOptionValue::SubCommand(cmd)
                | CommandDataOptionValue::SubCommandGroup(cmd) => {
                    let new_options = Options::new(cmd.clone());
                    return new_options.get_user(name);
                }
                CommandDataOptionValue::User(user) => {
                    if option.name == name {
                        return Some(User::from(*user));
                    }
                }
                _ => return None,
            }
        }
        None
    }

    pub fn get_role(&self, name: &str) -> Option<Role> {
        for option in &self.options {
            match &option.value {
                CommandDataOptionValue::SubCommand(cmd)
                | CommandDataOptionValue::SubCommandGroup(cmd) => {
                    let new_options = Options::new(cmd.clone());
                    return new_options.get_role(name);
                }
                CommandDataOptionValue::Role(role) => {
                    if option.name == name {
                        return Some(Role::from(*role));
                    }
                }
                _ => return None,
            }
        }
        None
    }

    pub fn get_string(&self, name: &str) -> Option<String> {
        for option in &self.options {
            match &option.value {
                CommandDataOptionValue::SubCommand(cmd)
                | CommandDataOptionValue::SubCommandGroup(cmd) => {
                    let new_options = Options::new(cmd.clone());
                    return new_options.get_string(name);
                }
                CommandDataOptionValue::String(string) => {
                    if option.name == name {
                        return Some(string.clone());
                    }
                }
                _ => return None,
            }
        }
        None
    }

    pub fn get_boolean(&self, name: &str) -> Option<bool> {
        for option in &self.options {
            match &option.value {
                CommandDataOptionValue::SubCommand(cmd)
                | CommandDataOptionValue::SubCommandGroup(cmd) => {
                    let new_options = Options::new(cmd.clone());
                    return new_options.get_boolean(name);
                }
                CommandDataOptionValue::Boolean(boolean) => {
                    if option.name == name {
                        return Some(*boolean);
                    }
                }
                _ => return None,
            }
        }
        None
    }

    pub fn get_integer(&self, name: &str) -> Option<i64> {
        for option in &self.options {
            match &option.value {
                CommandDataOptionValue::SubCommand(cmd)
                | CommandDataOptionValue::SubCommandGroup(cmd) => {
                    let new_options = Options::new(cmd.clone());
                    return new_options.get_integer(name);
                }
                CommandDataOptionValue::Integer(integer) => {
                    if option.name == name {
                        return Some(*integer);
                    }
                }
                _ => return None,
            }
        }
        None
    }
}
