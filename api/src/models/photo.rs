use sqlx::{FromRow, QueryBuilder, Row};

use crate::{bind, impl_media_common_ops};

use super::{DefaultRow, ReadOnlyPool, ReadWritePool};

pub struct Photo {
    pub processing: bool,
    pub processing_error: Option<String>,
    pub profile_picture: bool,
    pub banner: bool,
    pub jpg_small: Option<Vec<u8>>,
    pub jpg_medium: Option<Vec<u8>>,
    pub jpg_large: Option<Vec<u8>>,
}

impl FromRow<'_, DefaultRow> for Photo {
    fn from_row(row: &DefaultRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            processing: row.try_get::<i16, _>("processing")? == 1,
            processing_error: row.try_get("processing_error")?,
            profile_picture: row.try_get::<i16, _>("profile_picture")? == 1,
            banner: row.try_get::<i16, _>("banner")? == 1,
            jpg_small: row.try_get("jpg_small")?,
            jpg_medium: row.try_get("jpg_medium")?,
            jpg_large: row.try_get("jpg_large")?,
        })
    }
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

impl PhotoUpdateQuery {
    pub async fn update(&self, db: &ReadWritePool, id: i64) -> Result<(), sqlx::Error> {
        let mut builder = QueryBuilder::new("UPDATE photos SET ");
        let mut match_builder = builder.separated(", ");
        bind!(match_builder, self.processing as boolint);
        bind!(match_builder, self.processing_error);
        bind!(match_builder, self.profile_picture as boolint);
        bind!(match_builder, self.banner as boolint);
        bind!(match_builder, self.jpg_small);
        bind!(match_builder, self.jpg_medium);
        bind!(match_builder, self.jpg_large);
        builder.push(" WHERE id = ");
        builder.push_bind(id);
        builder.build().execute(&db.0).await?;
        Ok(())
    }
}

impl_media_common_ops!(
    Photo,
    "photos",
    "INSERT INTO photos (user_id, processing, processing_error, profile_picture, banner, jpg_small, jpg_medium, jpg_large) VALUES ($1, 1, NULL, 0, 0, NULL, NULL, NULL) RETURNING id"
);
