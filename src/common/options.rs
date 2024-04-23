use std::borrow::Cow;

use serenity::all::{ResolvedOption, ResolvedValue, Role, User};

#[derive(Debug, Clone)]
pub struct Options<'a> {
    pub options: Vec<ResolvedOption<'a>>,
}

impl Options<'_> {
    pub fn get_user(&self, name: &str) -> Cow<Option<User>> {
        for option in &self.options {
            match &option.value {
                ResolvedValue::SubCommandGroup(cmd) | ResolvedValue::SubCommand(cmd) => {
                    let sub_options = Options {
                        options: cmd.clone(),
                    };
                    let user = sub_options.get_user(name).into_owned().clone();
                    return Cow::Owned(user);
                }
                ResolvedValue::User(user, _) => {
                    if option.name == name {
                        return Cow::Owned(Some(user.to_owned().clone()));
                    }
                }
                _ => continue,
            }
        }
        Cow::Owned(None)
    }

    pub fn get_role(&self, name: &str) -> Cow<Option<Role>> {
        for option in &self.options {
            match &option.value {
                ResolvedValue::SubCommandGroup(cmd) | ResolvedValue::SubCommand(cmd) => {
                    let sub_options = Options {
                        options: cmd.clone(),
                    };
                    let role = sub_options.get_role(name).into_owned().clone();
                    return Cow::Owned(role);
                }
                ResolvedValue::Role(role) => {
                    if option.name == name {
                        return Cow::Owned(Some(role.to_owned().clone()));
                    }
                }
                _ => continue,
            }
        }
        Cow::Owned(None)
    }

    pub fn get_string(&self, name: &str) -> Cow<Option<String>> {
        for option in &self.options {
            match &option.value {
                ResolvedValue::SubCommandGroup(cmd) | ResolvedValue::SubCommand(cmd) => {
                    let sub_options = Options {
                        options: cmd.clone(),
                    };
                    let string = sub_options.get_string(name).into_owned().clone();
                    return Cow::Owned(string);
                }
                ResolvedValue::String(string) => {
                    if option.name == name {
                        return Cow::Owned(Some(string.to_owned().to_string()));
                    }
                }
                _ => continue,
            }
        }
        Cow::Owned(None)
    }

    pub fn get_boolean(&self, name: &str) -> Option<bool> {
        for option in &self.options {
            match &option.value {
                ResolvedValue::SubCommandGroup(cmd) | ResolvedValue::SubCommand(cmd) => {
                    let sub_options = Options {
                        options: cmd.clone(),
                    };
                    let boolean = sub_options.get_boolean(name);
                    return boolean;
                }
                ResolvedValue::Boolean(boolean) => {
                    if option.name == name {
                        return Some(boolean.to_owned());
                    }
                }
                _ => continue,
            }
        }
        None
    }

    pub fn get_integer(&self, name: &str) -> Option<i64> {
        for option in &self.options {
            match &option.value {
                ResolvedValue::SubCommandGroup(cmd) | ResolvedValue::SubCommand(cmd) => {
                    let sub_options = Options {
                        options: cmd.clone(),
                    };
                    let integer = sub_options.get_integer(name);
                    return integer;
                }
                ResolvedValue::Integer(integer) => {
                    if option.name == name {
                        return Some(integer.to_owned());
                    }
                }
                _ => continue,
            }
        }
        None
    }
}
