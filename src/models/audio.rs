use crate::cond;
use sqlx::{FromRow, Pool, Sqlite, sqlite::SqliteQueryResult};

#[derive(FromRow)]
pub struct Audio {
    pub processing: bool,
    pub processing_error: Option<String>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub thumbnail: Option<Vec<u8>>,
    pub mp3_128k: Option<Vec<u8>>,
}

#[derive(Default)]
pub struct AudioUpdateQuery {
    pub processing: Option<bool>,
    pub processing_error: Option<String>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub thumbnail: Option<Vec<u8>>,
    pub mp3_128k: Option<Vec<u8>>,
}

impl Audio {
    pub async fn insert(db: &Pool<Sqlite>, user_id: i64) -> Result<i64, sqlx::Error> {
        Ok(sqlx::query(
            "INSERT INTO audios (id, user_id, processing, processing_error, title, artist, thumbnail, mp3_128k) VALUES (NULL, $1, 1, NULL, NULL, NULL, NULL, NULL)",
        )
        .bind(user_id)
        .execute(db)
        .await?
        .last_insert_rowid())
    }

    pub async fn find(db: &Pool<Sqlite>, id: i64) -> Result<Audio, sqlx::Error> {
        sqlx::query_as("SELECT * FROM audios WHERE id = ?")
            .bind(id)
            .fetch_one(db)
            .await
    }

    pub async fn delete(db: &Pool<Sqlite>, id: i64) -> Result<SqliteQueryResult, sqlx::Error> {
        sqlx::query("DELETE FROM audios WHERE id = ?")
            .bind(id)
            .execute(db)
            .await
    }

    pub async fn update(
        db: &Pool<Sqlite>,
        id: i64,
        query: AudioUpdateQuery,
    ) -> Result<SqliteQueryResult, sqlx::Error> {
        let sql = format!("UPDATE audios SET {} WHERE id = $1", {
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
            cond!(query.mp3_128k.is_some(), append, "mp3_128k = $3");
            cond!(
                query.processing_error.is_some(),
                append,
                "processing_error = $4"
            );
            cond!(query.title.is_some(), append, "title = $5");
            cond!(query.artist.is_some(), append, "artist = $6");
            cond!(query.thumbnail.is_some(), append, "thumbnail = $7");
            clause
        });
        sqlx::query(&sql)
            .bind(id)
            .bind(query.processing)
            .bind(query.mp3_128k)
            .bind(query.processing_error)
            .bind(query.title)
            .bind(query.artist)
            .bind(query.thumbnail)
            .execute(db)
            .await
    }
}
