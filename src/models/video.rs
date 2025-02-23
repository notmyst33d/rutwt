use sqlx::{FromRow, Pool, Sqlite, sqlite::SqliteQueryResult};

use crate::cond;

#[derive(FromRow)]
pub struct Video {
    pub processing: bool,
    pub mp4_480p: Option<Vec<u8>>,
    pub mp4_720p: Option<Vec<u8>>,
}

#[derive(Default)]
pub struct VideoUpdateQuery {
    pub processing: Option<bool>,
    pub mp4_480p: Option<Vec<u8>>,
    pub mp4_720p: Option<Vec<u8>>,
}

impl Video {
    pub async fn insert(db: &Pool<Sqlite>, user_id: i64) -> Result<i64, sqlx::Error> {
        Ok(
            sqlx::query("INSERT INTO videos (id, user_id, processing, mp4_480p, mp4_720p) VALUES (NULL, $1, 1, NULL, NULL)")
                .bind(user_id)
                .execute(db)
                .await?
                .last_insert_rowid(),
        )
    }

    pub async fn find(db: &Pool<Sqlite>, id: i64) -> Result<Video, sqlx::Error> {
        sqlx::query_as("SELECT * FROM videos WHERE id = ?")
            .bind(id)
            .fetch_one(db)
            .await
    }

    pub async fn update(
        db: &Pool<Sqlite>,
        id: i64,
        query: VideoUpdateQuery,
    ) -> Result<SqliteQueryResult, sqlx::Error> {
        let sql = format!("UPDATE videos SET {} WHERE id = $1", {
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
            cond!(query.processing.is_some(), append, "processing = $2");
            cond!(query.mp4_480p.is_some(), append, "mp4_480p = $3");
            cond!(query.mp4_720p.is_some(), append, "mp4_720p = $4");
            clause
        });
        sqlx::query(&sql)
            .bind(id)
            .bind(query.processing)
            .bind(query.mp4_480p)
            .bind(query.mp4_720p)
            .execute(db)
            .await
    }
}
