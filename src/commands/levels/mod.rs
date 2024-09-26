use crate::models::command::Command;

pub mod leaderboard;
pub mod level;
pub mod rank;
pub mod xp;

pub fn get_level_commands() -> Vec<Box<dyn Command>> {
    vec![
        Box::new(leaderboard::LeaderboardCommand),
        Box::new(level::LevelCommand),
        Box::new(rank::RankCommand),
        Box::new(xp::XPCommand),
    ]
}
