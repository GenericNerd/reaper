use crate::models::{config::LoggingConfig, handler::Handler};

pub enum LogType {
    Action,
    Message,
    Voice,
}

pub async fn get_log_channel(
    handler: &Handler,
    logging_configuration: &LoggingConfig,
    log_type: &LogType,
) -> Option<i64> {
    let feature_flags =
        sqlx::query!("SELECT feature, active FROM global_kills WHERE feature ~ 'logging'")
            .fetch_all(&handler.main_database)
            .await
            .unwrap();

    if feature_flags
        .iter()
        .find(|flag| flag.feature == "logging")
        .unwrap()
        .active
    {
        return None;
    }

    match log_type {
        LogType::Action => {
            if feature_flags
                .iter()
                .find(|flag| flag.feature == "logging.action")
                .unwrap()
                .active
                || !logging_configuration.log_actions
            {
                return None;
            }
        }
        LogType::Message => {
            if feature_flags
                .iter()
                .find(|flag| flag.feature == "logging.message")
                .unwrap()
                .active
                || !logging_configuration.log_messages
            {
                return None;
            }
        }
        LogType::Voice => {
            if feature_flags
                .iter()
                .find(|flag| flag.feature == "logging.voice")
                .unwrap()
                .active
                || !logging_configuration.log_voice
            {
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
