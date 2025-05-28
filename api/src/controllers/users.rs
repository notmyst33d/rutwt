use std::sync::Arc;

use crate::{
    Claims, SharedState,
    controllers::auth::RESTRICTED_USERNAMES,
    errors::{
        CANNOT_DELETE_USER, CANNOT_FIND_USER, CANNOT_FOLLOW_SELF, CANNOT_INSERT_USER,
        CANNOT_UNFOLLOW_SELF, CANNOT_UPDATE_USER, USER_IS_ALREADY_FOLLOWED, USER_IS_NOT_FOLLOWED,
    },
    models::{User, user::UserUpdateQuery},
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use regex::Regex;

use super::{auth::USERNAME_REGEX, media::parse_media_id, posts::IdQuery};

#[derive(serde::Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub followers: i64,
    pub username: String,
    pub realname: String,
    pub bio: Option<String>,
    pub following: bool,
    pub profile_picture_photo_id: Option<String>,
    pub banner_photo_id: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UserSettingsRequest {
    pub username: Option<String>,
    pub realname: Option<String>,
    pub bio: Option<String>,
    pub profile_picture_photo_id: Option<String>,
    pub banner_photo_id: Option<String>,
}

async fn users_username(
    claims: Claims,
    State(state): State<Arc<SharedState>>,
    Path(username): Path<String>,
) -> axum::response::Result<impl IntoResponse> {
    Ok(Json::<UserResponse>(
        User::find(&state.db, None, Some(&username), Some(claims.user_id))
            .await
            .map_err(|_| (StatusCode::NOT_FOUND, CANNOT_FIND_USER))?
            .into(),
    ))
}

async fn users_self(
    claims: Claims,
    State(state): State<Arc<SharedState>>,
) -> axum::response::Result<impl IntoResponse> {
    Ok(Json::<UserResponse>(
        User::find(&state.db, Some(claims.user_id), None, Some(claims.user_id))
            .await
            .map_err(|_| (StatusCode::NOT_FOUND, CANNOT_FIND_USER))?
            .into(),
    ))
}

async fn users_follow(
    claims: Claims,
    State(state): State<Arc<SharedState>>,
    Query(query): Query<IdQuery>,
) -> axum::response::Result<impl IntoResponse> {
    if claims.user_id == query.id {
        return Err((StatusCode::BAD_REQUEST, CANNOT_FOLLOW_SELF).into());
    }
    if User::follow_exists(&state.db, claims.user_id, query.id).await {
        return Err((StatusCode::BAD_REQUEST, USER_IS_ALREADY_FOLLOWED).into());
    };
    User::follow_insert(&state.rwdb, claims.user_id, query.id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, CANNOT_INSERT_USER))?;
    Ok((StatusCode::OK, ""))
}

async fn users_unfollow(
    claims: Claims,
    State(state): State<Arc<SharedState>>,
    Query(query): Query<IdQuery>,
) -> axum::response::Result<impl IntoResponse> {
    if claims.user_id == query.id {
        return Err((StatusCode::BAD_REQUEST, CANNOT_UNFOLLOW_SELF).into());
    }
    if !User::follow_exists(&state.db, claims.user_id, query.id).await {
        return Err((StatusCode::BAD_REQUEST, USER_IS_NOT_FOLLOWED).into());
    };
    User::follow_delete(&state.rwdb, claims.user_id, query.id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, CANNOT_DELETE_USER))?;
    Ok((StatusCode::OK, ""))
}

async fn users_settings(
    claims: Claims,
    State(state): State<Arc<SharedState>>,
    Json(request): Json<UserSettingsRequest>,
) -> axum::response::Result<impl IntoResponse> {
    if request
        .realname
        .as_ref()
        .and_then(|v| Some(v.len() > 100 || v.len() == 0))
        .unwrap_or(false)
    {
        return Err((StatusCode::BAD_REQUEST, "uh oh").into());
    }

    if let Some(ref username) = request.username {
        if username.len() > 64 || username.len() < 3 {
            return Err((StatusCode::BAD_REQUEST, "no").into());
        }
        if RESTRICTED_USERNAMES.contains(&username.as_str()) {
            return Err((
                StatusCode::BAD_REQUEST,
                "user with this username already exists",
            )
                .into());
        }
        if !Regex::new(USERNAME_REGEX)
            .map_err(|_| (StatusCode::BAD_REQUEST, "uh oh regex"))?
            .is_match(&username)
            || username.contains("__")
            || username.starts_with("_")
        {
            return Err((StatusCode::BAD_REQUEST, "no").into());
        };
        let self_user = User::find(&state.db, Some(claims.user_id), None, None)
            .await
            .map_err(|_| (StatusCode::NOT_FOUND, CANNOT_FIND_USER))?;
        if User::find(&state.db, None, Some(&username), None)
            .await
            .ok()
            .map(|u| u.username != self_user.username)
            .unwrap_or_default()
        {
            return Err((
                StatusCode::BAD_REQUEST,
                "user with this username already exists",
            )
                .into());
        }
    }

    let mut query = UserUpdateQuery::default();
    query.realname = request.realname;
    query.username = request.username;
    query.bio = request.bio;
    query.profile_picture_photo_id = request
        .profile_picture_photo_id
        .and_then(|id| parse_media_id(&id).ok().and_then(|id| Some(id.1)));
    query.banner_photo_id = request
        .banner_photo_id
        .and_then(|id| parse_media_id(&id).ok().and_then(|id| Some(id.1)));

    User::update(&state.rwdb, claims.user_id, query)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, CANNOT_UPDATE_USER))?;

    Ok((StatusCode::OK, ""))
}

pub fn routes() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/", get(users_self))
        .route("/{username}", get(users_username))
        .route("/follow", get(users_follow))
        .route("/unfollow", get(users_unfollow))
        .route("/settings", post(users_settings))
}
