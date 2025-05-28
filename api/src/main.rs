mod controllers;
mod errors;
mod models;
mod services;
mod test;

use crate::models::{DefaultPool, ReadOnlyPool, ReadWritePool};
use axum::{
    RequestPartsExt, Router,
    extract::{DefaultBodyLimit, FromRequestParts},
    http::{StatusCode, request::Parts},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use chrono::{Duration, Utc};
use errors::UNAUTHORIZED;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use std::{error::Error, sync::Arc};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

#[macro_export]
macro_rules! cond {
    ($cond:expr, $func:tt, $sql:literal) => {
        if $cond {
            $func($sql);
        }
    };
}

#[derive(Clone)]
pub struct SharedState {
    pub db: ReadOnlyPool,
    pub rwdb: ReadWritePool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub user_id: i64,
    pub exp: i64,
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, UNAUTHORIZED))?;

        let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET not set");

        let token_data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| (StatusCode::UNAUTHORIZED, UNAUTHORIZED))?;

        Ok(token_data.claims)
    }
}

fn create_token(user_id: i64) -> Result<String, (StatusCode, &'static str)> {
    let claims = Claims {
        user_id,
        exp: (Utc::now() + Duration::days(1)).timestamp(),
    };

    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET not set");

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "cannot create new token"))
}

fn app(state: Arc<SharedState>) -> Router {
    Router::new()
        .nest("/api/posts", controllers::posts::routes())
        .nest("/api/auth", controllers::auth::routes())
        .nest("/api/media", controllers::media::routes())
        .nest("/api/users", controllers::users::routes())
        .layer(DefaultBodyLimit::max(256 * 1024 * 1024))
        .layer(CorsLayer::permissive())
        .with_state(state.clone())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(format!(
        "0.0.0.0:{}",
        std::env::var("PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(6969)
    ))
    .await?;

    axum::serve(
        listener,
        app(Arc::new(SharedState {
            db: ReadOnlyPool(
                DefaultPool::connect_lazy(
                    &std::env::var("READ_ONLY_DATABASE_URL")
                        .expect("READ_ONLY_DATABASE_URL not set"),
                )
                .unwrap(),
            ),
            rwdb: ReadWritePool(
                DefaultPool::connect_lazy(
                    &std::env::var("READ_WRITE_DATABASE_URL")
                        .expect("READ_WRITE_DATABASE_URL not set"),
                )
                .unwrap(),
            ),
        })),
    )
    .await?;

    Ok(())
}
