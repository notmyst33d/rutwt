pub mod audio;
pub mod photo;
pub mod post;
pub mod user;
pub mod video;

use std::ops::Deref;

pub use audio::Audio;
pub use photo::Photo;
pub use post::Post;
pub use user::User;
pub use video::Video;

#[cfg(feature = "postgres")]
pub type DefaultPool = sqlx::PgPool;
#[cfg(feature = "postgres")]
pub type DefaultRow = sqlx::postgres::PgRow;

#[cfg(any(feature = "sqlite", not(all(feature = "postgres"))))]
pub type DefaultPool = sqlx::SqlitePool;
#[cfg(any(feature = "sqlite", not(all(feature = "postgres"))))]
pub type DefaultRow = sqlx::sqlite::SqliteRow;

#[derive(Clone)]
pub struct ReadOnlyPool(pub DefaultPool);

impl Deref for ReadOnlyPool {
    type Target = DefaultPool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub struct ReadWritePool(pub DefaultPool);

impl Deref for ReadWritePool {
    type Target = DefaultPool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[macro_export]
macro_rules! bind {
    ($builder:ident, $self:ident.$field:ident) => {
        if $self.$field.is_some() {
            $builder.push(concat!(stringify!($field), " = "));
            $builder.push_bind_unseparated(&$self.$field);
        }
    };
    ($builder:ident, $self:ident.$field:ident as boolint) => {
        if $self.$field.is_some() {
            $builder.push(concat!(stringify!($field), " = "));
            $builder.push_bind_unseparated($self.$field.map(|v| if v == true { 1 } else { 0 }));
        }
    };
}

#[macro_export]
macro_rules! impl_media_common_ops {
    ($t:ty, $table:literal, $insert_sql:literal) => {
        impl $t {
            pub async fn insert(db: &ReadWritePool, user_id: i64) -> Result<i64, sqlx::Error> {
                sqlx::query_scalar($insert_sql)
                    .bind(user_id)
                    .fetch_one(&db.0)
                    .await
            }

            pub async fn find(db: &ReadOnlyPool, id: i64) -> Result<$t, sqlx::Error> {
                sqlx::query_as(concat!("SELECT * FROM ", $table, " WHERE id = $1"))
                    .bind(id)
                    .fetch_one(&db.0)
                    .await
            }

            pub async fn delete(db: &ReadWritePool, id: i64) -> Result<(), sqlx::Error> {
                sqlx::query(concat!("DELETE FROM ", $table, " WHERE id = $1"))
                    .bind(id)
                    .execute(&db.0)
                    .await?;
                Ok(())
            }
        }
    };
}
