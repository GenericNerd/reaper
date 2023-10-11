use std::borrow::Cow;

use serenity::all::{ResolvedOption, ResolvedValue, Role, User};

#[derive(Clone)]
pub struct Options<'a> {
    pub options: Vec<ResolvedOption<'a>>,
}

impl Options<'_> {
    pub fn get_user(&self, name: &str) -> Cow<Option<User>> {
        for option in &self.options {
            if let ResolvedValue::SubCommand(cmd) = &option.value {
                let sub_options = Options {
                    options: cmd.clone(),
                };
                let user = sub_options.get_user(name).into_owned().clone();
                return Cow::Owned(user);
            }
            if option.name == name {
                match &option.value {
                    ResolvedValue::User(user, _) => {
                        return Cow::Owned(Some(user.to_owned().clone()))
                    }
                    _ => continue,
                }
            }
        }
        Cow::Owned(None)
    }

    pub fn get_role(&self, name: &str) -> Cow<Option<Role>> {
        for option in &self.options {
            if let ResolvedValue::SubCommand(cmd) = &option.value {
                let sub_options = Options {
                    options: cmd.clone(),
                };
                let role = sub_options.get_role(name).into_owned().clone();
                return Cow::Owned(role);
            };
            if option.name == name {
                match &option.value {
                    ResolvedValue::Role(role) => return Cow::Owned(Some(role.to_owned().clone())),
                    _ => continue,
                }
            }
        }
        Cow::Owned(None)
    }

    pub fn get_string(&self, name: &str) -> Cow<Option<String>> {
        for option in &self.options {
            if let ResolvedValue::SubCommand(cmd) = &option.value {
                let sub_options = Options {
                    options: cmd.clone(),
                };
                let string = sub_options.get_string(name).into_owned().clone();
                return Cow::Owned(string);
            }
            if option.name == name {
                match &option.value {
                    ResolvedValue::String(string) => {
                        return Cow::Owned(Some(string.to_owned().to_string()))
                    }
                    _ => continue,
                }
            }
        }
        Cow::Owned(None)
    }

    pub fn get_boolean(&self, name: &str) -> Option<bool> {
        for option in &self.options {
            if let ResolvedValue::SubCommand(cmd) = &option.value {
                let sub_options = Options {
                    options: cmd.clone(),
                };
                let boolean = sub_options.get_boolean(name);
                return boolean;
            }
            if option.name == name {
                match &option.value {
                    ResolvedValue::Boolean(boolean) => {
                        return Some(boolean.to_owned());
                    }
                    _ => continue,
                }
            }
        }
        None
    }

    pub fn get_integer(&self, name: &str) -> Option<i64> {
        for option in &self.options {
            if let ResolvedValue::SubCommand(cmd) = &option.value {
                let sub_options = Options {
                    options: cmd.clone(),
                };
                let integer = sub_options.get_integer(name);
                return integer;
            }
            if option.name == name {
                match &option.value {
                    ResolvedValue::Integer(integer) => {
                        return Some(integer.to_owned());
                    }
                    _ => continue,
                }
            }
        }
        None
    }
}
