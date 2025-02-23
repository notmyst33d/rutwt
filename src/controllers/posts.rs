use crate::{
    Claims, SharedState,
    errors::{
        CANNOT_DELETE_POST, CANNOT_FIND_POST, CANNOT_INSERT_POST, CANNOT_USE_THIS_MEDIA_TYPE,
        MEDIA_NOT_FOUND, POST_IS_ALREADY_LIKED, POST_IS_NOT_LIKED,
    },
    models::Post,
};
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use std::sync::Arc;

use super::{
    media::{MediaType, parse_media_id},
    users::UserResponse,
};

#[derive(serde::Serialize, serde::Deserialize)]
struct PostRequest {
    message: Option<String>,
    media: Vec<String>,
    comment_post_id: Option<i64>,
}

#[derive(serde::Serialize)]
pub struct PostMedia {
    pub photo: Option<String>,
    pub video: Option<String>,
    pub audio: Option<String>,
}

#[derive(serde::Serialize)]
pub struct PostMention {
    user_id: i64,
    username: String,
}

#[derive(serde::Serialize)]
pub struct PostResponse {
    pub id: i64,
    pub message: Option<String>,
    pub like_count: i64,
    pub comment_count: i64,
    pub liked: bool,
    pub user: UserResponse,
    pub media: Vec<PostMedia>,
    pub mentions: Vec<PostMention>,
    pub comment: bool,
}

#[derive(serde::Serialize)]
pub struct PostTruncatedResponse {
    pub id: i64,
}

#[derive(serde::Deserialize)]
pub struct FindQuery {
    pub id: Option<i64>,
    pub username: Option<String>,
    pub offset: Option<i64>,
    pub count: Option<i64>,
}

#[derive(serde::Deserialize)]
pub struct CommentsQuery {
    pub id: i64,
    pub offset: Option<i64>,
    pub count: Option<i64>,
}

#[derive(serde::Deserialize)]
pub struct FeedQuery {
    pub offset: Option<i64>,
    pub count: Option<i64>,
}

#[derive(serde::Deserialize)]
pub struct IdQuery {
    pub id: i64,
}

async fn posts_like(
    claims: Claims,
    State(state): State<Arc<SharedState>>,
    Query(query): Query<IdQuery>,
) -> axum::response::Result<impl IntoResponse> {
    if Post::like_exists(&state.db, query.id, claims.user_id).await {
        return Err((StatusCode::BAD_REQUEST, POST_IS_ALREADY_LIKED).into());
    };
    Post::like_insert(&state.db, query.id, claims.user_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, CANNOT_INSERT_POST))?;
    Ok((StatusCode::OK, ""))
}

async fn posts_unlike(
    claims: Claims,
    State(state): State<Arc<SharedState>>,
    Query(query): Query<IdQuery>,
) -> axum::response::Result<impl IntoResponse> {
    if !Post::like_exists(&state.db, query.id, claims.user_id).await {
        return Err((StatusCode::BAD_REQUEST, POST_IS_NOT_LIKED).into());
    };
    Post::like_delete(&state.db, query.id, claims.user_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, CANNOT_DELETE_POST))?;
    Ok((StatusCode::OK, ""))
}

async fn posts_find(
    claims: Claims,
    State(state): State<Arc<SharedState>>,
    Query(query): Query<FindQuery>,
) -> axum::response::Result<impl IntoResponse> {
    let posts = Post::find(
        &state.db,
        query.offset.unwrap_or(0),
        query.count.unwrap_or(100),
        query.id,
        query.username.as_deref(),
        None,
        false,
        false,
        claims.user_id,
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, CANNOT_FIND_POST))?;

    Ok(Json(
        posts
            .into_iter()
            .map(|p| p.into_response())
            .collect::<Vec<_>>(),
    ))
}

async fn posts_comments(
    claims: Claims,
    State(state): State<Arc<SharedState>>,
    Query(query): Query<CommentsQuery>,
) -> axum::response::Result<impl IntoResponse> {
    let posts = Post::find(
        &state.db,
        query.offset.unwrap_or(0),
        query.count.unwrap_or(100),
        Some(query.id),
        None,
        None,
        false,
        true,
        claims.user_id,
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, CANNOT_FIND_POST))?;

    Ok(Json(
        posts
            .into_iter()
            .map(|p| p.into_response())
            .collect::<Vec<_>>(),
    ))
}

async fn posts_feed(
    claims: Claims,
    State(state): State<Arc<SharedState>>,
    Query(query): Query<FeedQuery>,
) -> axum::response::Result<impl IntoResponse> {
    let posts = Post::find(
        &state.db,
        query.offset.unwrap_or(0),
        query.count.unwrap_or(100),
        None,
        None,
        Some(claims.user_id),
        true,
        false,
        claims.user_id,
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, CANNOT_FIND_POST))?;

    Ok(Json(
        posts
            .into_iter()
            .map(|p| p.into_response())
            .collect::<Vec<_>>(),
    ))
}

async fn posts_create(
    claims: Claims,
    State(state): State<Arc<SharedState>>,
    Json(request): Json<PostRequest>,
) -> axum::response::Result<impl IntoResponse> {
    if request.media.len() > 5
        || request
            .message
            .as_ref()
            .and_then(|v| Some(v.len() > 2048))
            .unwrap_or(false)
    {
        return Err((StatusCode::BAD_REQUEST, "uh oh").into());
    }

    if (request.message.is_none()
        || request
            .message
            .as_deref()
            .and_then(|m| Some(m == ""))
            .unwrap_or_default())
        && request.media.len() == 0
    {
        return Err((StatusCode::BAD_REQUEST, "uh oh").into());
    }

    if let Some(comment_post_id) = request.comment_post_id {
        if !Post::exists(&state.db, comment_post_id).await {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, CANNOT_FIND_POST).into());
        }
    }

    let id = Post::insert(
        &state.db,
        claims.user_id,
        request.message.as_deref(),
        request.comment_post_id.is_some(),
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, CANNOT_INSERT_POST))?;

    for media_id in request.media {
        let (media_type, media_inner_id) = parse_media_id(&media_id)?;
        match media_type {
            MediaType::Photo => Post::photo_insert(&state.db, id, media_inner_id).await,
            MediaType::Video => Post::video_insert(&state.db, id, media_inner_id).await,
            MediaType::Audio => Post::audio_insert(&state.db, id, media_inner_id).await,
            MediaType::ProfilePicture | MediaType::Banner => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    CANNOT_USE_THIS_MEDIA_TYPE,
                )
                    .into());
            }
        }
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, MEDIA_NOT_FOUND))?;
    }

    if let Some(comment_post_id) = request.comment_post_id {
        Post::comment_insert(&state.db, comment_post_id, claims.user_id, id)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, CANNOT_INSERT_POST))?;
    }

    Ok(Json(PostTruncatedResponse { id }))
}

pub fn routes() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/create", post(posts_create))
        .route("/like", get(posts_like))
        .route("/unlike", get(posts_unlike))
        .route("/find", get(posts_find))
        .route("/comments", get(posts_comments))
        .route("/feed", get(posts_feed))
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use crate::{
        controllers::posts::PostRequest,
        test::instrumentation::{init, send_post},
    };

    #[tokio::test]
    async fn post() {
        let (state, token) = init().await;
        let response = send_post(state.clone(), "/api/posts/create", Some(&token), &PostRequest {
            message: Some("test".to_string()),
            media: vec![],
            comment_post_id: None,
        })
        .await;
        assert!(response.status() == StatusCode::OK);
    }
}
