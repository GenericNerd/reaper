use std::borrow::Cow;

use serenity::all::{ResolvedOption, ResolvedValue, Role, User};

#[derive(Clone)]
pub struct Options<'a> {
    pub options: Vec<ResolvedOption<'a>>,
}

impl Options<'_> {
    pub fn get_user(&self, name: &str) -> Cow<Option<User>> {
        for option in self.options.iter() {
            if option.name == name {
                match &option.value {
                    ResolvedValue::SubCommand(cmd) => {
                        let sub_options = Options {
                            options: cmd.to_owned(),
                        };
                        let user = sub_options.get_user(name).into_owned().clone();
                        return Cow::Owned(user);
                    }
                    ResolvedValue::User(user, _) => {
                        return Cow::Owned(Some(user.to_owned().to_owned()));
                    }
                    _ => continue,
                }
            }
        }
        Cow::Owned(None)
    }

    pub fn get_role(&self, name: &str) -> Cow<Option<Role>> {
        for option in self.options.iter() {
            if option.name == name {
                match &option.value {
                    ResolvedValue::SubCommand(cmd) => {
                        let sub_options = Options {
                            options: cmd.to_owned(),
                        };
                        let role = sub_options.get_role(name).into_owned().clone();
                        return Cow::Owned(role);
                    }
                    ResolvedValue::Role(role) => {
                        return Cow::Owned(Some(role.to_owned().to_owned()));
                    }
                    _ => continue,
                }
            }
        }
        Cow::Owned(None)
    }
}
