#[cfg(test)]
pub mod instrumentation {
    use std::sync::Arc;

    use axum::{
        body::Body,
        http::{Request, header},
        response::Response,
    };
    use http_body_util::BodyExt;
    use serde::{Deserialize, Serialize};
    use sqlx::SqlitePool;
    use tower::ServiceExt;

    use crate::{
        SharedState, app,
        controllers::auth::{RegisterRequest, UserMixedAuthResponse},
    };

    pub async fn send_post<T: Serialize>(
        state: Arc<SharedState>,
        uri: &str,
        token: Option<&str>,
        body: &T,
    ) -> Response<Body> {
        let mut builder = Request::builder()
            .method("POST")
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json");
        if let Some(token) = token {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        let request = builder
            .body(Body::from(serde_json::to_string(body).unwrap()))
            .unwrap();
        app(state).oneshot(request).await.unwrap()
    }

    pub async fn init() -> (Arc<SharedState>, String) {
        unsafe { std::env::set_var("JWT_SECRET", "test") };

        let state = Arc::new(SharedState {
            db: SqlitePool::connect(":memory:").await.unwrap(),
        });

        sqlx::query(include_str!("../data/0000-base-schema.sql"))
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query(include_str!("../data/0001-media-update.sql"))
            .execute(&state.db)
            .await
            .unwrap();

        let response = send_post(
            state.clone(),
            "/api/auth/register",
            None,
            &RegisterRequest {
                realname: "test".to_string(),
                username: "test".to_string(),
                password: "test".to_string(),
            },
        )
        .await;
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let data: UserMixedAuthResponse = serde_json::from_slice(&bytes).unwrap();
        (state, data.token)
    }

    pub async fn json<T: for<'a> Deserialize<'a>>(body: Response<Body>) -> T {
        serde_json::from_slice(&body.into_body().collect().await.unwrap().to_bytes()).unwrap()
    }
}
