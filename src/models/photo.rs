use crate::cond;
use sqlx::{FromRow, Pool, Sqlite, sqlite::SqliteQueryResult};

#[derive(FromRow)]
pub struct Photo {
    pub processing: bool,
    pub processing_error: Option<String>,
    pub profile_picture: bool,
    pub banner: bool,
    pub jpg_small: Option<Vec<u8>>,
    pub jpg_medium: Option<Vec<u8>>,
    pub jpg_large: Option<Vec<u8>>,
}

#[derive(Default)]
pub struct PhotoUpdateQuery {
    pub processing: Option<bool>,
    pub processing_error: Option<String>,
    pub profile_picture: Option<bool>,
    pub banner: Option<bool>,
    pub jpg_small: Option<Vec<u8>>,
    pub jpg_medium: Option<Vec<u8>>,
    pub jpg_large: Option<Vec<u8>>,
}

impl Photo {
    pub async fn insert(db: &Pool<Sqlite>, user_id: i64) -> Result<i64, sqlx::Error> {
        Ok(sqlx::query("INSERT INTO photos (id, user_id, processing, processing_error, profile_picture, banner, jpg_small, jpg_medium, jpg_large) VALUES (NULL, $1, 1, NULL, 0, 0, NULL, NULL, NULL)")
            .bind(user_id)
            .execute(db)
            .await?.last_insert_rowid())
    }

    pub async fn find(db: &Pool<Sqlite>, id: i64) -> Result<Photo, sqlx::Error> {
        sqlx::query_as("SELECT * FROM photos WHERE id = ?")
            .bind(id)
            .fetch_one(db)
            .await
    }

    pub async fn delete(db: &Pool<Sqlite>, id: i64) -> Result<SqliteQueryResult, sqlx::Error> {
        sqlx::query("DELETE FROM photos WHERE id = ?")
            .bind(id)
            .execute(db)
            .await
    }

    pub async fn update(
        db: &Pool<Sqlite>,
        id: i64,
        query: PhotoUpdateQuery,
    ) -> Result<SqliteQueryResult, sqlx::Error> {
        let sql = format!("UPDATE photos SET {} WHERE id = $1", {
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
            cond!(query.banner.is_some(), append, "banner = $2");
            cond!(query.processing.is_some(), append, "processing = $3");
            cond!(
                query.profile_picture.is_some(),
                append,
                "profile_picture = $4"
            );

            cond!(query.jpg_small.is_some(), append, "jpg_small = $5");
            cond!(query.jpg_medium.is_some(), append, "jpg_medium = $6");
            cond!(query.jpg_large.is_some(), append, "jpg_large = $7");
            cond!(
                query.processing_error.is_some(),
                append,
                "processing_error = $8"
            );
            clause
        });
        sqlx::query(&sql)
            .bind(id)
            .bind(query.banner)
            .bind(query.processing)
            .bind(query.profile_picture)
            .bind(query.jpg_small)
            .bind(query.jpg_medium)
            .bind(query.jpg_large)
            .bind(query.processing_error)
            .execute(db)
            .await
    }
}
