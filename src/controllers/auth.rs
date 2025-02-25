use std::sync::Arc;

use crate::{
    SharedState, create_token,
    errors::{CANNOT_INSERT_USER, INVALID_USERNAME_OR_PASSWORD},
    models::user::User,
};
use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use regex::Regex;

pub const USERNAME_REGEX: &'static str = "^[a-zA-Z0-9_]+$";
pub const RESTRICTED_USERNAMES: [&'static str; 6] =
    ["settings", "login", "register", "latest", "post", "api"];

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct RegisterRequest {
    pub realname: String,
    pub username: String,
    pub password: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UserMixedAuthResponse {
    pub token: String,
}

async fn auth_login(
    State(state): State<Arc<SharedState>>,
    Json(request): Json<LoginRequest>,
) -> axum::response::Result<impl IntoResponse> {
    let user = User::find(&state.db, None, Some(&request.username), None)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, INVALID_USERNAME_OR_PASSWORD))?;

    // TODO: Костыли...
    if user.hashed_password != request.password {
        return Err((StatusCode::UNAUTHORIZED, INVALID_USERNAME_OR_PASSWORD).into());
    }

    Ok(Json(UserMixedAuthResponse {
        token: create_token(user.id)?,
    }))
}

async fn auth_register(
    State(state): State<Arc<SharedState>>,
    Json(request): Json<RegisterRequest>,
) -> axum::response::Result<impl IntoResponse> {
    if request.password.len() < 8 {
        return Err((StatusCode::BAD_REQUEST, "password too short").into());
    }
    if (request.realname.len() == 0 || request.realname.len() > 100)
        || (request.username.len() > 64 || request.username.len() < 3)
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "username or real name invalid length",
        )
            .into());
    }
    if RESTRICTED_USERNAMES.contains(&request.username.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            "user with this username already exists",
        )
            .into());
    }
    if !Regex::new(USERNAME_REGEX)
        .map_err(|_| (StatusCode::BAD_REQUEST, "uh oh"))?
        .is_match(&request.username)
        || request.username.contains("__")
        || request.username.starts_with("_")
    {
        return Err((StatusCode::BAD_REQUEST, "no").into());
    };

    if User::find(&state.db, None, Some(&request.username), None)
        .await
        .is_ok()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "user with this username already exists",
        )
            .into());
    }

    let user_id = User::insert(
        &state.db,
        &request.username,
        &request.realname,
        &request.password,
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, CANNOT_INSERT_USER))?;

    Ok(Json(UserMixedAuthResponse {
        token: create_token(user_id)?,
    }))
}

pub fn routes() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/login", post(auth_login))
        .route("/register", post(auth_register))
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use crate::{
        controllers::auth::{LoginRequest, RegisterRequest, UserMixedAuthResponse},
        test::instrumentation::{init, json, send_post},
    };

    #[tokio::test]
    async fn register_and_login() {
        let (state, _) = init().await;
        let response = send_post(
            state.clone(),
            "/api/auth/register",
            None,
            &RegisterRequest {
                realname: "test2".to_string(),
                username: "test2".to_string(),
                password: "test2test2test2".to_string(),
            },
        )
        .await;
        assert!(response.status() == StatusCode::OK);

        let data = json::<UserMixedAuthResponse>(response).await;
        assert!(data.token.len() != 0);

        let response = send_post(state.clone(), "/api/auth/login", None, &LoginRequest {
            username: "test2".to_string(),
            password: "test2test2test2".to_string(),
        })
        .await;
        assert!(response.status() == StatusCode::OK);

        let data = json::<UserMixedAuthResponse>(response).await;
        assert!(data.token.len() != 0);
    }
}
