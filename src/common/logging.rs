use crate::models::config::LoggingConfig;

// TODO: Remove when used
pub enum LogType {
    Action,
    #[allow(dead_code)]
    Message,
    #[allow(dead_code)]
    Voice,
}

pub fn get_log_channel(logging_configuration: &LoggingConfig, log_type: &LogType) -> Option<i64> {
    match log_type {
        LogType::Action => {
            if !logging_configuration.log_actions {
                return None;
            }
        }
        LogType::Message => {
            if !logging_configuration.log_messages {
                return None;
            }
        }
        LogType::Voice => {
            if !logging_configuration.log_voice {
                return None;
            }
        }
    }

    if logging_configuration.log_channel.is_some() {
        logging_configuration.log_channel
    } else {
        match log_type {
            LogType::Action => logging_configuration.log_action_channel,
            LogType::Message => logging_configuration.log_message_channel,
            LogType::Voice => logging_configuration.log_voice_channel,
        }
    }
}
