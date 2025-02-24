use sqlx::{FromRow, Pool, Sqlite, sqlite::SqliteQueryResult};

use crate::cond;

#[derive(FromRow)]
pub struct Video {
    pub processing: bool,
    pub processing_error: Option<String>,
    pub thumbnail: Option<Vec<u8>>,
    pub mp4_480p: Option<Vec<u8>>,
}

#[derive(Default)]
pub struct VideoUpdateQuery {
    pub processing: Option<bool>,
    pub processing_error: Option<String>,
    pub thumbnail: Option<Vec<u8>>,
    pub mp4_480p: Option<Vec<u8>>,
}

impl Video {
    pub async fn insert(db: &Pool<Sqlite>, user_id: i64) -> Result<i64, sqlx::Error> {
        Ok(
            sqlx::query("INSERT INTO videos (id, user_id, processing, processing_error, thumbnail, mp4_480p) VALUES (NULL, $1, 1, NULL, NULL, NULL)")
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

    pub async fn delete(db: &Pool<Sqlite>, id: i64) -> Result<SqliteQueryResult, sqlx::Error> {
        sqlx::query("DELETE FROM videos WHERE id = ?")
            .bind(id)
            .execute(db)
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
            cond!(query.thumbnail.is_some(), append, "thumbnail = $3");
            cond!(query.mp4_480p.is_some(), append, "mp4_480p = $4");
            cond!(
                query.processing_error.is_some(),
                append,
                "processing_error = $5"
            );
            clause
        });
        sqlx::query(&sql)
            .bind(id)
            .bind(query.processing)
            .bind(query.thumbnail)
            .bind(query.mp4_480p)
            .bind(query.processing_error)
            .execute(db)
            .await
    }
}
