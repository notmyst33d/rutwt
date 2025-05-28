use crate::controllers::{
    media::{MediaType, encode_media_id},
    posts::{PostMedia, PostMediaAudio, PostResponse},
    users::UserResponse,
};
use serde::{Deserialize, Deserializer};
use sqlx::{FromRow, QueryBuilder, Row};

use super::{DefaultRow, ReadOnlyPool, ReadWritePool};

macro_rules! media_insert {
    ($name:literal, $fnname:ident, $idname:ident) => {
        pub async fn $fnname(
            db: &ReadWritePool,
            post_id: i64,
            $idname: i64,
        ) -> Result<(), sqlx::Error> {
            sqlx::query(concat!(
                "INSERT INTO posts_",
                $name,
                "s (post_id, ",
                $name,
                "_id) VALUES ($1, $2)"
            ))
            .bind(post_id)
            .bind($idname)
            .execute(&db.0)
            .await?;
            Ok(())
        }
    };
}

fn from_int_bool<'a, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'a>,
{
    let value: Option<u8> = Deserialize::deserialize(deserializer)?;
    Ok(value.map(|v| v == 1).unwrap_or_default())
}

#[derive(Debug, serde::Deserialize)]
pub struct PostAudio {
    pub id: i64,
    pub title: Option<String>,
    pub artist: Option<String>,
    #[serde(deserialize_with = "from_int_bool")]
    pub thumbnail: bool,
}

#[derive(Debug)]
pub struct Post {
    pub post_id: i64,
    pub post_message: Option<String>,
    pub post_like_count: i64,
    pub post_comment_count: i64,
    pub post_photos: Vec<i64>,
    pub post_videos: Vec<i64>,
    pub post_audios: Vec<PostAudio>,
    pub post_comment: bool,
    pub post_liked: bool,
    pub user_id: i64,
    pub user_followers: i64,
    pub user_username: String,
    pub user_realname: String,
    pub user_bio: Option<String>,
    pub user_following: bool,
    pub user_profile_picture_photo_id: Option<i64>,
    pub user_banner_photo_id: Option<i64>,
}

#[derive(Default)]
pub struct PostFindQuery {
    pub offset: i64,
    pub count: i64,
    pub comments: bool,
    pub feed: bool,
    pub self_user_id: i64,
    pub id: Option<i64>,
    pub username: Option<String>,
}

impl Post {
    pub async fn insert(
        db: &ReadWritePool,
        user_id: i64,
        message: Option<&str>,
        comment: bool,
    ) -> Result<i64, sqlx::Error> {
        Ok(sqlx::query_scalar(
            "INSERT INTO posts (user_id, message, comment, deleted) VALUES ($1, $2, cast($3 as INTEGER), 0) RETURNING id",
        )
        .bind(user_id)
        .bind(message)
        .bind(comment)
        .fetch_one(&db.0)
        .await?)
    }

    pub async fn exists(db: &ReadOnlyPool, id: i64) -> bool {
        sqlx::query("SELECT * FROM posts WHERE id = $1")
            .bind(id)
            .fetch_one(&db.0)
            .await
            .map_or(false, |_| true)
    }

    pub async fn find(db: &ReadOnlyPool, query: PostFindQuery) -> Result<Vec<Post>, sqlx::Error> {
        let mut builder = QueryBuilder::new(
            "
        SELECT
            posts.id AS post_id,
            posts.message AS post_message,
            posts.comment AS post_comment,
            (SELECT concat('[', string_agg(cast(photo_id as TEXT), ','), ']') FROM posts_photos WHERE post_id = posts.id) AS post_photos,
            (SELECT concat('[', string_agg(cast(video_id as TEXT), ','), ']') FROM posts_videos WHERE post_id = posts.id) AS post_videos,
            (
                SELECT
                    concat('[', string_agg(concat('{\"id\":', audios.id, ',\"title\":\"', replace(audios.title, '\"', '\\\"'), '\",\"artist\":\"', replace(audios.artist, '\"', '\\\"'), '\",\"thumbnail\":', cast(audios.thumbnail IS NOT NULL as INTEGER), '}'), ','), ']')
                FROM posts_audios
                INNER JOIN audios ON audios.id = posts_audios.audio_id
                WHERE post_id = posts.id
            ) AS post_audios,
            (SELECT COUNT(*) FROM likes WHERE post_id = posts.id) AS post_like_count,
            (SELECT COUNT(*) FROM comments WHERE post_id = posts.id) AS post_comment_count,
            (SELECT COUNT(*) FROM likes WHERE post_id = posts.id AND user_id = $5) AS post_liked,
            users.id AS user_id,
            users.username AS user_username,
            users.realname AS user_realname,
            users.bio AS user_bio,
            users.profile_picture_photo_id AS user_profile_picture_photo_id,
            users.banner_photo_id AS user_banner_photo_id,
            (SELECT COUNT(*) FROM follows WHERE user_id = $5 AND sub_user_id = users.id) AS user_following,
            (SELECT COUNT(*) FROM follows WHERE sub_user_id = users.id) AS user_followers
        FROM posts
        INNER JOIN users ON users.id = posts.user_id
        WHERE
        ");

        let mut match_builder = builder.separated(" AND ");
        if query.id.is_some() && query.comments {
            match_builder
                .push("posts.id IN (SELECT comment_post_id FROM comments WHERE post_id = $3)");
        } else if query.id.is_some() {
            match_builder.push("posts.id = $3");
        } else if !query.comments {
            match_builder.push("posts.comment = 0");
        } else if query.comments {
            match_builder.push("posts.comment = 1");
        }
        if query.username.is_some() {
            match_builder.push("users.username = $4");
        }
        if query.feed {
            match_builder.push("users.id IN (SELECT sub_user_id FROM follows WHERE user_id = $5)");
        }
        match_builder.push("users.deleted = 0");
        match_builder.push("posts.deleted = 0");
        builder.push(" ORDER BY posts.id DESC LIMIT $2 OFFSET $1");
        builder
            .build_query_as()
            .bind(query.offset)
            .bind(query.count)
            .bind(query.id)
            .bind(query.username)
            .bind(query.self_user_id)
            .fetch_all(&db.0)
            .await
    }

    pub async fn like_insert(
        db: &ReadWritePool,
        post_id: i64,
        user_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO likes (post_id, user_id) VALUES ($1, $2)")
            .bind(post_id)
            .bind(user_id)
            .execute(&db.0)
            .await?;
        Ok(())
    }

    pub async fn comment_insert(
        db: &ReadWritePool,
        post_id: i64,
        user_id: i64,
        comment_post_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO comments (post_id, user_id, comment_post_id) VALUES ($1, $2, $3)")
            .bind(post_id)
            .bind(user_id)
            .bind(comment_post_id)
            .execute(&db.0)
            .await?;
        Ok(())
    }

    pub async fn like_exists(db: &ReadOnlyPool, post_id: i64, user_id: i64) -> bool {
        sqlx::query("SELECT * FROM likes WHERE post_id = $1 AND user_id = $2")
            .bind(post_id)
            .bind(user_id)
            .fetch_optional(&db.0)
            .await
            .and_then(|v| Ok(v.is_some()))
            .unwrap_or(false)
    }

    pub async fn like_delete(
        db: &ReadWritePool,
        post_id: i64,
        user_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM likes WHERE post_id = $1 AND user_id = $2")
            .bind(post_id)
            .bind(user_id)
            .execute(&db.0)
            .await?;
        Ok(())
    }

    media_insert!("photo", photo_insert, photo_id);
    media_insert!("video", video_insert, video_id);
    media_insert!("audio", audio_insert, audio_id);
}

impl FromRow<'_, DefaultRow> for Post {
    fn from_row(row: &DefaultRow) -> Result<Self, sqlx::Error> {
        let post_photos: Vec<i64> = serde_json::from_str(row.try_get("post_photos")?).unwrap();
        let post_videos: Vec<i64> = serde_json::from_str(row.try_get("post_videos")?).unwrap();
        let post_audios: Vec<PostAudio> =
            serde_json::from_str(row.try_get("post_audios")?).unwrap();
        Ok(Self {
            post_id: row.try_get("post_id")?,
            post_message: row.try_get("post_message")?,
            post_like_count: row.try_get("post_like_count")?,
            post_comment_count: row.try_get("post_comment_count")?,
            post_photos,
            post_videos,
            post_audios,
            post_comment: row.try_get::<i16, _>("post_comment")? == 1,
            post_liked: row.try_get::<i64, _>("post_liked")? == 1,
            user_id: row.try_get("user_id")?,
            user_followers: row.try_get("user_followers")?,
            user_username: row.try_get("user_username")?,
            user_realname: row.try_get("user_realname")?,
            user_bio: row.try_get("user_bio")?,
            user_following: row.try_get::<i64, _>("user_following")? == 1,
            user_profile_picture_photo_id: row.try_get("user_profile_picture_photo_id")?,
            user_banner_photo_id: row.try_get("user_banner_photo_id")?,
        })
    }
}

impl Into<PostResponse> for Post {
    fn into(self) -> PostResponse {
        let mut media = vec![];
        self.post_photos.iter().for_each(|p| {
            media.push(PostMedia {
                photo: Some(encode_media_id(MediaType::Photo, *p)),
                video: None,
                audio: None,
            })
        });
        self.post_videos.iter().for_each(|p| {
            media.push(PostMedia {
                photo: None,
                video: Some(encode_media_id(MediaType::Video, *p)),
                audio: None,
            })
        });
        self.post_audios.iter().for_each(|p| {
            media.push(PostMedia {
                photo: None,
                video: None,
                audio: Some(PostMediaAudio {
                    id: encode_media_id(MediaType::Audio, p.id),
                    title: p.title.clone(),
                    artist: p.artist.clone(),
                    thumbnail: p.thumbnail,
                }),
            })
        });
        PostResponse {
            id: self.post_id,
            message: self.post_message,
            like_count: self.post_like_count,
            comment_count: self.post_comment_count,
            liked: self.post_liked,
            user: UserResponse {
                id: self.user_id,
                followers: self.user_followers,
                username: self.user_username,
                realname: self.user_realname,
                bio: self.user_bio,
                following: self.user_following,
                profile_picture_photo_id: self
                    .user_profile_picture_photo_id
                    .and_then(|id| Some(encode_media_id(MediaType::ProfilePicture, id))),
                banner_photo_id: self
                    .user_banner_photo_id
                    .and_then(|id| Some(encode_media_id(MediaType::Banner, id))),
            },
            media,
            mentions: vec![],
            comment: self.post_comment,
        }
    }
}
