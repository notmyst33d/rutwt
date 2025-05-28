use sqlx::{FromRow, Row};

use crate::{
    cond,
    controllers::{
        media::{MediaType, encode_media_id},
        users::UserResponse,
    },
};

use super::{DefaultRow, ReadOnlyPool, ReadWritePool};

pub struct User {
    pub id: i64,
    pub username: String,
    pub realname: String,
    pub bio: Option<String>,
    pub hashed_password: String,
    pub profile_picture_photo_id: Option<i64>,
    pub banner_photo_id: Option<i64>,
    pub followers: i64,
    pub following: bool,
}

#[derive(Default)]
#[allow(dead_code)]
pub struct UserUpdateQuery {
    pub username: Option<String>,
    pub realname: Option<String>,
    pub bio: Option<String>,
    pub hashed_password: Option<String>,
    pub profile_picture_photo_id: Option<i64>,
    pub banner_photo_id: Option<i64>,
}

impl User {
    pub async fn insert(
        db: &ReadWritePool,
        username: &str,
        realname: &str,
        hashed_password: &str,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar("INSERT INTO users (username, realname, hashed_password, profile_picture_photo_id, banner_photo_id, bio, deleted) VALUES ($1, $2, $3, NULL, NULL, NULL, 0) RETURNING id")
            .bind(username)
            .bind(realname)
            .bind(hashed_password)
            .fetch_one(&db.0)
            .await
    }

    pub async fn find(
        db: &ReadOnlyPool,
        id: Option<i64>,
        username: Option<&str>,
        self_user_id: Option<i64>,
    ) -> Result<User, sqlx::Error> {
        let sql = format!(
            "
        SELECT
            *,
            (SELECT COUNT(*) FROM follows WHERE sub_user_id = id) AS followers,
            (SELECT COUNT(*) FROM follows WHERE user_id = $3 AND sub_user_id = id) AS following
        FROM users
        {}
        ",
            {
                let mut first = true;
                let mut clause = String::new();
                let mut append = |cond| {
                    clause += if first {
                        first = false;
                        " WHERE "
                    } else {
                        " AND "
                    };
                    clause += cond;
                };
                cond!(id.is_some(), append, "id = $1");
                cond!(username.is_some(), append, "username = $2");
                append("deleted = 0");
                clause
            }
        );
        sqlx::query_as(&sql)
            .bind(id)
            .bind(username)
            .bind(self_user_id)
            .fetch_one(&db.0)
            .await
    }

    pub async fn update(
        db: &ReadWritePool,
        id: i64,
        query: UserUpdateQuery,
    ) -> Result<(), sqlx::Error> {
        let sql = format!("UPDATE users SET {} WHERE id = $1", {
            let mut first = true;
            let mut clause = String::new();
            let mut append = |cond| {
                if first {
                    first = false;
                    clause += cond;
                } else {
                    clause += ", ";
                    clause += cond;
                };
            };
            cond!(query.realname.is_some(), append, "realname = $2");
            cond!(query.username.is_some(), append, "username = $3");
            cond!(query.bio.is_some(), append, "bio = $4");
            cond!(
                query.profile_picture_photo_id.is_some(),
                append,
                "profile_picture_photo_id = $5"
            );
            cond!(
                query.banner_photo_id.is_some(),
                append,
                "banner_photo_id = $6"
            );
            clause
        });
        sqlx::query(&sql)
            .bind(id)
            .bind(query.realname)
            .bind(query.username)
            .bind(query.bio)
            .bind(query.profile_picture_photo_id)
            .bind(query.banner_photo_id)
            .execute(&db.0)
            .await?;
        Ok(())
    }

    pub async fn follow_insert(
        db: &ReadWritePool,
        user_id: i64,
        sub_user_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO follows (user_id, sub_user_id) VALUES ($1, $2)")
            .bind(user_id)
            .bind(sub_user_id)
            .execute(&db.0)
            .await?;
        Ok(())
    }

    pub async fn follow_exists(db: &ReadOnlyPool, user_id: i64, sub_user_id: i64) -> bool {
        sqlx::query("SELECT * FROM follows WHERE user_id = $1 AND sub_user_id = $2")
            .bind(user_id)
            .bind(sub_user_id)
            .fetch_one(&db.0)
            .await
            .map_or(false, |_| true)
    }

    pub async fn follow_delete(
        db: &ReadWritePool,
        user_id: i64,
        sub_user_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM follows WHERE user_id = $1 AND sub_user_id = $2")
            .bind(user_id)
            .bind(sub_user_id)
            .execute(&db.0)
            .await?;
        Ok(())
    }
}

impl FromRow<'_, DefaultRow> for User {
    fn from_row(row: &DefaultRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            username: row.try_get("username")?,
            realname: row.try_get("realname")?,
            bio: row.try_get("bio")?,
            hashed_password: row.try_get("hashed_password")?,
            profile_picture_photo_id: row.try_get("profile_picture_photo_id")?,
            banner_photo_id: row.try_get("banner_photo_id")?,
            followers: row.try_get("followers")?,
            following: row.try_get::<i64, _>("following")? == 1,
        })
    }
}

impl Into<UserResponse> for User {
    fn into(self) -> UserResponse {
        UserResponse {
            id: self.id,
            followers: self.followers,
            username: self.username,
            realname: self.realname,
            bio: self.bio,
            following: self.following,
            profile_picture_photo_id: self
                .profile_picture_photo_id
                .and_then(|id| Some(encode_media_id(MediaType::ProfilePicture, id))),
            banner_photo_id: self
                .banner_photo_id
                .and_then(|id| Some(encode_media_id(MediaType::Banner, id))),
        }
    }
}
