use futures::TryStreamExt;
use sqlite2pg::Migrate;
use sqlx::{PgPool, QueryBuilder, SqlitePool, prelude::FromRow};

#[derive(FromRow, Migrate)]
#[table("posts")]
struct Post {
    #[seq_key]
    id: i64,
    user_id: i64,
    message: Option<String>,
    comment: i16,
    deleted: i16,
}

#[derive(FromRow, Migrate)]
#[table("likes")]
struct Like {
    post_id: i64,
    user_id: i64,
}

#[derive(FromRow, Migrate)]
#[table("comments")]
struct Comment {
    post_id: i64,
    user_id: i64,
    comment_post_id: i64,
}

#[derive(FromRow, Migrate)]
#[table("reposts")]
struct Repost {
    post_id: i64,
    user_id: i64,
}

#[derive(FromRow, Migrate)]
#[table("posts_photos")]
struct PostPhoto {
    post_id: i64,
    photo_id: i64,
}

#[derive(FromRow, Migrate)]
#[table("posts_videos")]
struct PostVideo {
    post_id: i64,
    video_id: i64,
}

#[derive(FromRow, Migrate)]
#[table("posts_audios")]
struct PostAudio {
    post_id: i64,
    audio_id: i64,
}

#[derive(FromRow, Migrate)]
#[table("users")]
struct User {
    #[seq_key]
    id: i64,
    username: String,
    realname: String,
    hashed_password: String,
    profile_picture_photo_id: Option<i64>,
    banner_photo_id: Option<i64>,
    bio: Option<String>,
    deleted: i16,
}

#[derive(FromRow, Migrate)]
#[table("follows")]
struct Follow {
    user_id: i64,
    sub_user_id: i64,
}

#[derive(FromRow, Migrate)]
#[table("photos")]
struct Photo {
    #[seq_key]
    id: i64,
    user_id: i64,
    processing: i16,
    processing_error: Option<String>,
    profile_picture: i16,
    banner: i16,
    jpg_small: Option<Vec<u8>>,
    jpg_medium: Option<Vec<u8>>,
    jpg_large: Option<Vec<u8>>,
}

#[derive(FromRow, Migrate)]
#[table("videos")]
struct Video {
    #[seq_key]
    id: i64,
    user_id: i64,
    processing: i16,
    processing_error: Option<String>,
    thumbnail: Option<Vec<u8>>,
    mp4_480p: Option<Vec<u8>>,
}

#[derive(FromRow, Migrate)]
#[table("audios")]
struct Audio {
    #[seq_key]
    id: i64,
    user_id: i64,
    processing: i16,
    processing_error: Option<String>,
    title: Option<String>,
    artist: Option<String>,
    mp3_128k: Option<Vec<u8>>,
    thumbnail: Option<Vec<u8>>,
}

#[tokio::main]
async fn main() {
    let postgres =
        PgPool::connect_lazy(&std::env::var("POSTGRES_URL").expect("POSTGRES_URL not set"))
            .unwrap();

    let sqlite =
        SqlitePool::connect_lazy(&std::env::var("SQLITE_URL").expect("SQLITE_URL not set"))
            .unwrap();

    Post::migrate(&sqlite, &postgres).await;
    Like::migrate(&sqlite, &postgres).await;
    Comment::migrate(&sqlite, &postgres).await;
    Repost::migrate(&sqlite, &postgres).await;
    PostPhoto::migrate(&sqlite, &postgres).await;
    PostVideo::migrate(&sqlite, &postgres).await;
    PostAudio::migrate(&sqlite, &postgres).await;
    User::migrate(&sqlite, &postgres).await;
    Follow::migrate(&sqlite, &postgres).await;
    Photo::migrate(&sqlite, &postgres).await;
    Video::migrate(&sqlite, &postgres).await;
    Audio::migrate(&sqlite, &postgres).await;
}
