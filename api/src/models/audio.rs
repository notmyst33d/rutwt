use sqlx::{FromRow, QueryBuilder, Row};

use crate::{bind, impl_media_common_ops};

use super::{DefaultRow, ReadOnlyPool, ReadWritePool};

pub struct Audio {
    pub processing: bool,
    pub processing_error: Option<String>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub thumbnail: Option<Vec<u8>>,
    pub mp3_128k: Option<Vec<u8>>,
}

impl FromRow<'_, DefaultRow> for Audio {
    fn from_row(row: &DefaultRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            processing: row.try_get::<i16, _>("processing")? == 1,
            processing_error: row.try_get("processing_error")?,
            title: row.try_get("title")?,
            artist: row.try_get("artist")?,
            thumbnail: row.try_get("thumbnail")?,
            mp3_128k: row.try_get("mp3_128k")?,
        })
    }
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

impl AudioUpdateQuery {
    pub async fn update(&self, db: &ReadWritePool, id: i64) -> Result<(), sqlx::Error> {
        let mut builder = QueryBuilder::new("UPDATE audios SET ");
        let mut match_builder = builder.separated(", ");
        bind!(match_builder, self.processing as boolint);
        bind!(match_builder, self.processing_error);
        bind!(match_builder, self.title);
        bind!(match_builder, self.artist);
        bind!(match_builder, self.thumbnail);
        bind!(match_builder, self.mp3_128k);
        builder.push(" WHERE id = ");
        builder.push_bind(id);
        builder.build().execute(&db.0).await?;
        Ok(())
    }
}

impl_media_common_ops!(
    Audio,
    "audios",
    "INSERT INTO audios (user_id, processing, processing_error, title, artist, thumbnail, mp3_128k) VALUES ($1, 1, NULL, NULL, NULL, NULL, NULL) RETURNING id"
);
