use serenity::all::{CommandDataOption, CommandDataOptionValue, CommandInteraction};

use crate::models::{role::Role, user::User};

pub struct Options {
    options: Vec<CommandDataOption>,
}

impl Options {
    fn find<T>(
        options: &[CommandDataOption],
        name: &str,
        f: impl Fn(&CommandDataOptionValue) -> Option<T>,
    ) -> Option<T> {
        let mut stack: Vec<&[CommandDataOption]> = vec![options];
        while let Some(opts) = stack.pop() {
            for opt in opts {
                match &opt.value {
                    CommandDataOptionValue::SubCommand(cmd)
                    | CommandDataOptionValue::SubCommandGroup(cmd) => stack.push(cmd),
                    v if opt.name == name => {
                        if let Some(val) = f(v) {
                            return Some(val);
                        }
                    }
                    _ => {}
                }
            }
        }
        None
    }

    pub fn from_command(cmd: &CommandInteraction) -> Self {
        Self {
            options: cmd.data.options.clone(),
        }
    }

    pub fn get_user(&self, name: &str) -> Option<User> {
        Self::find(&self.options, name, |v| match v {
            CommandDataOptionValue::User(user) => Some(User::from(*user)),
            _ => None,
        })
    }

    pub fn get_role(&self, name: &str) -> Option<Role> {
        Self::find(&self.options, name, |v| match v {
            CommandDataOptionValue::Role(role) => Some(Role::from(*role)),
            _ => None,
        })
    }

    pub fn get_string(&self, name: &str) -> Option<String> {
        Self::find(&self.options, name, |v| match v {
            CommandDataOptionValue::String(string) => Some(string.clone()),
            _ => None,
        })
    }

    pub fn get_boolean(&self, name: &str) -> Option<bool> {
        Self::find(&self.options, name, |v| match v {
            CommandDataOptionValue::Boolean(boolean) => Some(*boolean),
            _ => None,
        })
    }

    pub fn get_integer(&self, name: &str) -> Option<i64> {
        Self::find(&self.options, name, |v| match v {
            CommandDataOptionValue::Integer(integer) => Some(*integer),
            _ => None,
        })
    }
}
