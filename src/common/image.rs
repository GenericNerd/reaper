use image::load_from_memory;
use image_builder::{colors, FilterType, Image, Picture, Rect, Text};
use lazy_static::lazy_static;
use serenity::all::Member;

use crate::models::{handler::Handler, response::ResponseError};

const IMAGE_WIDTH: u32 = 800;
const IMAGE_HEIGHT: u32 = 200;

fn get_number_formatter() -> numfmt::Formatter {
    numfmt::Formatter::default()
        .precision(numfmt::Precision::Significance(3))
        .scales(
            numfmt::Scales::new(1000, vec!["", "K", "M", "B", "T", "P", "E", "Z", "Y"]).unwrap(),
        )
}

fn number_to_string(number: i64) -> String {
    if number < 1000 {
        return number.to_string();
    }
    get_number_formatter().fmt2(number).to_string()
}

impl Handler {
    pub async fn generate_image(&self, member: Member) -> Result<String, ResponseError> {
        let user_id = member.user.id.get() as i64;
        let guild_id = member.guild_id.get() as i64;

        let avatar_hash = match member.user.avatar {
            Some(avatar_hash) => avatar_hash.to_string(),
            None => ((user_id >> 22) % 6).to_string(),
        };
        let avatar_url = match member.user.avatar {
            Some(avatar_hash) => {
                format!("https://cdn.discordapp.com/avatars/{user_id}/{avatar_hash}.png?size=1024",)
            }
            None => {
                format!("https://cdn.discordapp.com/avatars/{user_id}/{avatar_hash}.png?size=1024",)
            }
        };

        let reqwest_client = reqwest::Client::new();
        let Ok(avatar_response) = reqwest_client.get(&avatar_url).send().await else {
            return Err(ResponseError::Execution(
                "Could not get avatar",
                Some(format!("Failed to get image from {avatar_url}")),
            ));
        };
        let Ok(avatar_bytes) = avatar_response.bytes().await else {
            return Err(ResponseError::Execution(
                "Could not get avatar",
                Some(format!("Failed to get image from {avatar_url}")),
            ));
        };
        let avatar_bytes = avatar_bytes.to_vec();
        let Ok(avatar) = load_from_memory(&avatar_bytes) else {
            return Err(ResponseError::Execution(
                "Could not get avatar",
                Some(format!("Failed to get image from {avatar_url}")),
            ));
        };

        let mut image = Image::new(IMAGE_WIDTH, IMAGE_HEIGHT, [44, 47, 52, 255]);
        lazy_static! {
            static ref JETBRAINS_REGULAR: Vec<u8> =
                std::fs::read("JetBrainsMono-Regular.ttf").unwrap();
            static ref JETBRAINS_SEMIBOLD: Vec<u8> =
                std::fs::read("JetBrainsMono-SemiBold.ttf").unwrap();
            static ref JETBRAINS_BOLD: Vec<u8> = std::fs::read("JetBrainsMono-Bold.ttf").unwrap();
        };
        image.add_custom_font("JetBrains Regular", JETBRAINS_REGULAR.to_vec());
        image.add_custom_font("JetBrains SemiBold", JETBRAINS_SEMIBOLD.to_vec());
        image.add_custom_font("JetBrains Bold", JETBRAINS_BOLD.to_vec());

        image.add_rect(
            Rect::new()
                .position(5, 5)
                .size(IMAGE_HEIGHT - 10, IMAGE_HEIGHT - 10)
                .color(colors::WHITE),
        );

        image.add_picture(
            Picture::new(avatar)
                .resize(IMAGE_HEIGHT - 20, IMAGE_HEIGHT - 20, FilterType::Triangle)
                .position(10, 10),
        );

        image.add_text(
            Text::new(member.user.name.as_str())
                .size(40)
                .position(IMAGE_HEIGHT + 10, 5)
                .color(colors::WHITE)
                .font("JetBrains Bold"),
        );

        image.add_rect(
            Rect::new()
                .size(IMAGE_WIDTH - IMAGE_HEIGHT - 20 - 10, 2)
                .position(IMAGE_HEIGHT + 10, 45)
                .color([199, 89, 66, 255]),
        );

        let xp_info = sqlx::query!(
            "SELECT xp, rank FROM (SELECT user_id, xp, RANK() OVER (ORDER BY xp DESC) AS rank FROM user_xp WHERE guild_id = $1) ranked_users WHERE user_id = $2",
            guild_id,
            user_id
        )
        .fetch_one(&self.main_database)
        .await?;

        let current_level =
            ((-25.0 + f64::sqrt((625 + (200 * xp_info.xp)) as f64)) / 100.0).floor() as i64;
        let next_level = current_level + 1;
        let current_level_xp = (50 * (current_level * current_level)) + (25 * current_level);
        let next_level_xp = (50 * (next_level * next_level)) + (25 * next_level) - current_level_xp;
        let progress_to_next_level = (xp_info.xp - current_level_xp) as f32 / next_level_xp as f32;

        image.add_text(
            Text::new(&format!("Level: {current_level}"))
                .size(40)
                .position(IMAGE_HEIGHT + 10, 50)
                .color(colors::WHITE)
                .font("JetBrains SemiBold"),
        );

        image.add_text(
            Text::new(&format!(
                "Rank: {}",
                number_to_string(xp_info.rank.unwrap())
            ))
            .size(35)
            .position(IMAGE_WIDTH - 175, 90)
            .color(colors::WHITE)
            .font("JetBrains Regular"),
        );

        image.add_text(
            Text::new(&format!(
                "XP: {}/{}",
                number_to_string(xp_info.xp - current_level_xp),
                number_to_string(next_level_xp)
            ))
            .size(35)
            .position(IMAGE_HEIGHT + 10, 90)
            .color(colors::WHITE)
            .font("JetBrains Regular"),
        );

        image.add_rect(
            Rect::new()
                .size(IMAGE_WIDTH - IMAGE_HEIGHT - 20, IMAGE_HEIGHT / 3)
                .position(IMAGE_HEIGHT + 5, IMAGE_HEIGHT - (IMAGE_HEIGHT / 3) - 5)
                .color(colors::WHITE),
        );

        if progress_to_next_level > 0_f32 {
            image.add_rect(
                Rect::new()
                    .size(
                        ((IMAGE_WIDTH - IMAGE_HEIGHT - 20 - 10) as f32 * progress_to_next_level)
                            .round() as u32,
                        IMAGE_HEIGHT / 3 - 10,
                    )
                    .position(IMAGE_HEIGHT + 10, IMAGE_HEIGHT - (IMAGE_HEIGHT / 3))
                    .color([235, 151, 109, 255]),
            );
        }

        let filename = &format!("{user_id}_{avatar_hash}.png");
        image.save(filename);
        Ok(filename.to_string())
    }
}
