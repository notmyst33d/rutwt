use sqlx::{FromRow, QueryBuilder, Row};

use crate::{bind, impl_media_common_ops};

use super::{DefaultRow, ReadOnlyPool, ReadWritePool};

pub struct Video {
    pub processing: bool,
    pub processing_error: Option<String>,
    pub thumbnail: Option<Vec<u8>>,
    pub mp4_480p: Option<Vec<u8>>,
}

impl FromRow<'_, DefaultRow> for Video {
    fn from_row(row: &DefaultRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            processing: row.try_get::<i16, _>("processing")? == 1,
            processing_error: row.try_get("processing_error")?,
            thumbnail: row.try_get("thumbnail")?,
            mp4_480p: row.try_get("mp4_480p")?,
        })
    }
}

#[derive(Default)]
pub struct VideoUpdateQuery {
    pub processing: Option<bool>,
    pub processing_error: Option<String>,
    pub thumbnail: Option<Vec<u8>>,
    pub mp4_480p: Option<Vec<u8>>,
}

impl VideoUpdateQuery {
    pub async fn update(&self, db: &ReadWritePool, id: i64) -> Result<(), sqlx::Error> {
        let mut builder = QueryBuilder::new("UPDATE videos SET ");
        let mut match_builder = builder.separated(", ");
        bind!(match_builder, self.processing as boolint);
        bind!(match_builder, self.processing_error);
        bind!(match_builder, self.thumbnail);
        bind!(match_builder, self.mp4_480p);
        builder.push(" WHERE id = ");
        builder.push_bind(id);
        builder.build().execute(&db.0).await?;
        Ok(())
    }
}

impl_media_common_ops!(
    Video,
    "videos",
    "INSERT INTO videos (user_id, processing, processing_error, thumbnail, mp4_480p) VALUES ($1, 1, NULL, NULL, NULL) RETURNING id"
);
