use image::{load_from_memory, DynamicImage};
use image_builder::{colors, FilterType, Image, Picture, Rect, Text};
use lazy_static::lazy_static;
use serenity::all::{Context, Member, User, UserId};

use crate::models::{handler::Handler, response::ResponseError};

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

fn get_url_from_user(user: &User) -> String {
    let user_id = user.id.get() as i64;
    let avatar_hash = match user.avatar {
        Some(avatar_hash) => avatar_hash.to_string(),
        None => ((user_id >> 22) % 6).to_string(),
    };
    match user.avatar {
        Some(avatar_hash) => {
            format!("https://cdn.discordapp.com/avatars/{user_id}/{avatar_hash}.png?size=1024")
        }
        None => {
            format!("https://cdn.discordapp.com/avatars/{user_id}/{avatar_hash}.png?size=1024")
        }
    }
}

impl Handler {
    pub async fn generate_rank_image(&self, member: Member) -> Result<String, ResponseError> {
        const IMAGE_WIDTH: u32 = 800;
        const IMAGE_HEIGHT: u32 = 200;

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

        let progress_width =
            ((IMAGE_WIDTH - IMAGE_HEIGHT - 20 - 10) as f32 * progress_to_next_level).round() as u32;
        if progress_width > 0 {
            image.add_rect(
                Rect::new()
                    .size(progress_width, IMAGE_HEIGHT / 3 - 10)
                    .position(IMAGE_HEIGHT + 10, IMAGE_HEIGHT - (IMAGE_HEIGHT / 3))
                    .color([235, 151, 109, 255]),
            );
        }

        let filename = &format!("{user_id}_{avatar_hash}.png");
        image.save(filename);
        Ok(filename.to_string())
    }

    pub async fn generate_leaderboard_image(
        &self,
        ctx: &Context,
        guild_id: i64,
    ) -> Result<String, ResponseError> {
        #[derive(Clone)]
        struct User {
            name: String,
            logo: DynamicImage,
            xp: i64,
        }
        const IMAGE_WIDTH: u32 = 720;

        let leaderboard_info = sqlx::query!(
            "SELECT user_id, xp FROM user_xp WHERE guild_id = $1 ORDER BY xp DESC LIMIT 10",
            guild_id
        )
        .fetch_all(&self.main_database)
        .await?;

        let mut leaderboard_users = vec![];
        for leaderboard_record in leaderboard_info {
            let Ok(user) = ctx
                .http
                .get_user(UserId::new(leaderboard_record.user_id as u64))
                .await
            else {
                return Err(ResponseError::Execution(
                    "Failed to fetch user",
                    Some("Failed to fetch user".to_string()),
                ));
            };

            let avatar_url = get_url_from_user(&user);
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
            leaderboard_users.push(User {
                name: user.name,
                logo: avatar,
                xp: leaderboard_record.xp,
            });
        }

        let lb_member_count = leaderboard_users.len();
        let image_height = (lb_member_count as u32 * 110)
            + if lb_member_count < 3 {
                (lb_member_count * 10) as u32
            } else {
                25
            };

        let mut image = Image::new(IMAGE_WIDTH, image_height, [44, 47, 52, 255]);
        lazy_static! {
            static ref JETBRAINS_REGULAR: Vec<u8> =
                std::fs::read("JetBrainsMono-Regular.ttf").unwrap();
            static ref JETBRAINS_SEMIBOLD: Vec<u8> =
                std::fs::read("JetBrainsMono-SemiBold.ttf").unwrap();
            static ref JETBRAINS_BOLD: Vec<u8> = std::fs::read("JetBrainsMono-Bold.ttf").unwrap();
        };
        image.add_custom_font("JetBrains Regular", JETBRAINS_REGULAR.to_vec());
        image.add_custom_font("JetBrains Bold", JETBRAINS_BOLD.to_vec());
        image.add_custom_font("JetBrains SemiBold", JETBRAINS_SEMIBOLD.to_vec());

        for (i, user) in leaderboard_users.iter().enumerate() {
            let user = user.clone();

            image.add_rect(
                Rect::new()
                    .size(
                        match i {
                            0..=2 => 110,
                            _ => 95,
                        },
                        match i {
                            0..=2 => 110,
                            _ => 95,
                        },
                    )
                    .position(
                        5,
                        if i < 3 {
                            (i as u32 * 120) + 5
                        } else {
                            (i as u32 * 110) + 35
                        },
                    )
                    .color(match i {
                        0 => [255, 215, 0, 255],
                        1 => [192, 192, 192, 255],
                        2 => [205, 127, 50, 255],
                        _ => [35, 36, 40, 255],
                    }),
            );

            image.add_picture(
                Picture::new(user.logo)
                    .position(
                        10,
                        if i < 3 {
                            (i as u32 * 120) + 10
                        } else {
                            (i as u32 * 110) + 40
                        },
                    )
                    .resize(
                        match i {
                            0..=2 => 100,
                            _ => 85,
                        },
                        match i {
                            0..=2 => 100,
                            _ => 85,
                        },
                        FilterType::Triangle,
                    ),
            );

            if i != lb_member_count - 1 {
                image.add_rect(
                    Rect::new()
                        .size(IMAGE_WIDTH, 4)
                        .position(
                            0,
                            if i < 3 {
                                (i as u32 * 120) + 118
                            } else {
                                135 + (i as u32 * 110)
                            },
                        )
                        .color([199, 89, 66, 255]),
                );
            }

            image.add_text(
                Text::new(user.name.as_str())
                    .size(40)
                    .position(
                        if i < 3 { 120 } else { 105 },
                        if i < 3 {
                            (i as u32 * 120) + 75
                        } else {
                            95 + (i as u32 * 110)
                        },
                    )
                    .color(colors::WHITE)
                    .font("JetBrains SemiBold"),
            );
            image.add_text(
                Text::new(&format!("#{}", i + 1))
                    .size(80)
                    .position(
                        if i < 3 { 120 } else { 105 },
                        if i < 3 {
                            (i as u32 * 120) + 5
                        } else {
                            30 + (i as u32 * 110)
                        },
                    )
                    .color(colors::WHITE)
                    .font("JetBrains Bold"),
            );

            let level =
                ((-25.0 + f64::sqrt((625 + (200 * user.xp)) as f64)) / 100.0).floor() as i64;
            image.add_text(
                Text::new(&format!("Level {}", number_to_string(level)))
                    .size(60)
                    .position(
                        400,
                        if i < 3 {
                            (i as u32 * 120) + 15
                        } else {
                            40 + (i as u32 * 110)
                        },
                    )
                    .color(colors::WHITE)
                    .font("JetBrains Regular"),
            );
        }

        image.save(&format!("{guild_id}.png"));

        Ok(format!("{guild_id}.png"))
    }
}
