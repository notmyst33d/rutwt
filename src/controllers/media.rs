use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Multipart, Path, State},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};

use crate::{
    Claims, SharedState,
    errors::{CANNOT_USE_THIS_MEDIA_TYPE, INVALID_MEDIA_ID, MEDIA_IS_PROCESSING, MEDIA_NOT_FOUND},
    models::{
        Audio, Photo, Video, audio::AudioUpdateQuery, photo::PhotoUpdateQuery,
        video::VideoUpdateQuery,
    },
    services::media,
};

enum MediaIdVersion {
    V1 = 1,
}

#[derive(PartialEq)]
pub enum MediaType {
    Photo = 1,
    Video = 2,
    Audio = 3,
    ProfilePicture = 4,
    Banner = 5,
}

#[derive(serde::Serialize)]
struct MediaResponse {
    pub id: String,
    pub processing: bool,
}

pub fn encode_media_id(media_type: MediaType, id: i64) -> String {
    let mut buf = vec![];
    buf.extend_from_slice(&(MediaIdVersion::V1 as i8).to_le_bytes());
    buf.extend_from_slice(&(media_type as i8).to_le_bytes());
    buf.extend_from_slice(&id.to_le_bytes());
    BASE64_URL_SAFE_NO_PAD.encode(&buf)
}

pub fn parse_media_id(id: &str) -> Result<(MediaType, i64), (StatusCode, &'static str)> {
    let data = BASE64_URL_SAFE_NO_PAD
        .decode(&id)
        .map_err(|_| (StatusCode::BAD_REQUEST, INVALID_MEDIA_ID))?;
    if data.len() < 1 {
        return Err((StatusCode::BAD_REQUEST, INVALID_MEDIA_ID));
    }
    let version = match data[0] {
        1 => MediaIdVersion::V1,
        _ => return Err((StatusCode::BAD_REQUEST, INVALID_MEDIA_ID)),
    };
    match version {
        MediaIdVersion::V1 => {
            if data.len() < 10 {
                return Err((StatusCode::BAD_REQUEST, INVALID_MEDIA_ID));
            }
            let mut x = [0u8; 8];
            let Some(id_data) = data.get(2..) else {
                return Err((StatusCode::BAD_REQUEST, INVALID_MEDIA_ID));
            };
            x.clone_from_slice(&id_data);

            Ok((
                match data[1] {
                    1 => MediaType::Photo,
                    2 => MediaType::Video,
                    3 => MediaType::Audio,
                    4 => MediaType::ProfilePicture,
                    5 => MediaType::Banner,
                    _ => return Err((StatusCode::BAD_REQUEST, INVALID_MEDIA_ID)),
                },
                i64::from_le_bytes(x),
            ))
        }
    }
}

async fn media_handler(
    State(state): State<Arc<SharedState>>,
    Path(id): Path<String>,
) -> axum::response::Result<impl IntoResponse> {
    let Some((id, ext)) = id.split_once(".") else {
        return Err((StatusCode::BAD_REQUEST, CANNOT_USE_THIS_MEDIA_TYPE).into());
    };

    let (media_type, num_id) = parse_media_id(id)?;
    let mut headers = HeaderMap::new();

    if ext.len() < 3 {
        return Err((StatusCode::BAD_REQUEST, CANNOT_USE_THIS_MEDIA_TYPE).into());
    }

    let (media, filename) = match &ext[..3] {
        "jpg" => {
            headers.insert(header::CONTENT_TYPE, "image/jpeg".parse().unwrap());
            let default_res = if media_type == MediaType::ProfilePicture {
                "small"
            } else {
                "medium"
            };
            let (_, res) = ext.split_once(":").unwrap_or((ext, default_res));
            let photo = Photo::find(&state.db, num_id)
                .await
                .map_err(|_| (StatusCode::NOT_FOUND, MEDIA_NOT_FOUND))?;
            if photo.processing {
                return Err((StatusCode::NO_CONTENT, MEDIA_IS_PROCESSING).into());
            }
            if (photo.profile_picture && media_type == MediaType::Photo)
                || (!photo.profile_picture && media_type == MediaType::ProfilePicture)
            {
                return Err((StatusCode::BAD_REQUEST, CANNOT_USE_THIS_MEDIA_TYPE).into());
            }
            if (photo.banner && media_type == MediaType::Photo)
                || (!photo.profile_picture && media_type == MediaType::ProfilePicture)
            {
                return Err((StatusCode::BAD_REQUEST, CANNOT_USE_THIS_MEDIA_TYPE).into());
            }
            let (media, actual_res) = match res {
                "small" => (photo.jpg_small, "small"),
                "medium" => (photo.jpg_medium, "medium"),
                "large" => (photo.jpg_large, "large"),
                "orig" => (photo.jpg_orig, "orig"),
                _ => {
                    if media_type == MediaType::ProfilePicture {
                        (photo.jpg_small, default_res)
                    } else {
                        (photo.jpg_medium, default_res)
                    }
                }
            };
            (media.unwrap(), format!("{id}_{actual_res}.jpg"))
        }
        "mp4" => {
            headers.insert(header::CONTENT_TYPE, "video/mp4".parse().unwrap());
            let (_, res) = ext.split_once(":").unwrap_or((ext, "480p"));
            let video = Video::find(&state.db, num_id)
                .await
                .map_err(|_| (StatusCode::NOT_FOUND, MEDIA_NOT_FOUND))?;
            if video.processing {
                return Err((StatusCode::NO_CONTENT, MEDIA_IS_PROCESSING).into());
            }
            let (media, actual_res) = match res {
                "720p" => (video.mp4_720p, "720p"),
                _ => (video.mp4_480p, "480p"),
            };
            (media.unwrap(), format!("{id}_{actual_res}.mp4"))
        }
        "mp3" => {
            headers.insert(header::CONTENT_TYPE, "audio/mpeg".parse().unwrap());
            let (_, res) = ext.split_once(":").unwrap_or((ext, "128k"));
            let audio = Audio::find(&state.db, num_id)
                .await
                .map_err(|_| (StatusCode::NOT_FOUND, MEDIA_NOT_FOUND))?;
            if audio.processing {
                return Err((StatusCode::NO_CONTENT, MEDIA_IS_PROCESSING).into());
            }
            let (media, actual_res) = match res {
                "320k" => (audio.mp3_320k, "320k"),
                _ => (audio.mp3_128k, "128k"),
            };
            (media.unwrap(), format!("{id}_{actual_res}.mp3"))
        }
        _ => return Err((StatusCode::BAD_REQUEST, CANNOT_USE_THIS_MEDIA_TYPE).into()),
    };

    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("inline; filename=\"{filename}\"").parse().unwrap(),
    );

    Ok((headers, media))
}

async fn media_upload(
    State(state): State<Arc<SharedState>>,
    claims: Claims,
    mut multipart: Multipart,
) -> axum::response::Result<impl IntoResponse> {
    let mut media_type = None;
    let mut media_data = None;

    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        match name.as_str() {
            "type" => media_type = Some(String::from_utf8(data.to_vec()).unwrap()),
            "data" => media_data = Some(data.to_vec()),
            _ => {}
        }
    }

    let Some(media_type) = media_type else {
        return Err((StatusCode::BAD_REQUEST, "cannot find media type").into());
    };

    let Some(media_data) = media_data else {
        return Err((StatusCode::BAD_REQUEST, "cannot find data").into());
    };

    let id = match media_type.as_str() {
        "photo" => {
            let id = Photo::insert(&state.db, claims.user_id).await.unwrap();
            tokio::spawn(async move {
                let result = media::process_photo(media_data).await.unwrap();
                let mut query = PhotoUpdateQuery::default();
                query.processing = Some(false);
                query.jpg_small = Some(result.jpg_small);
                query.jpg_medium = Some(result.jpg_medium);
                query.jpg_large = Some(result.jpg_large);
                query.jpg_orig = Some(result.jpg_orig);
                Photo::update(&state.db, id, query).await.unwrap();
            });
            encode_media_id(MediaType::Photo, id)
        }
        "video" => {
            let id = Video::insert(&state.db, claims.user_id).await.unwrap();
            tokio::spawn(async move {
                let result = media::process_video(media_data).await.unwrap();
                let mut query = VideoUpdateQuery::default();
                query.processing = Some(false);
                query.mp4_480p = Some(result.mp4_480p);
                query.mp4_720p = Some(result.mp4_720p);
                Video::update(&state.db, id, query).await.unwrap();
            });
            encode_media_id(MediaType::Video, id)
        }
        "audio" => {
            let id = Audio::insert(&state.db, claims.user_id).await.unwrap();
            tokio::spawn(async move {
                let result = media::process_audio(media_data).await.unwrap();
                let mut query = AudioUpdateQuery::default();
                query.processing = Some(false);
                query.mp3_128k = Some(result.mp3_128k);
                query.mp3_320k = Some(result.mp3_320k);
                Audio::update(&state.db, id, query).await.unwrap();
            });
            encode_media_id(MediaType::Audio, id)
        }
        "profile_picture" => {
            let id = Photo::insert(&state.db, claims.user_id).await.unwrap();
            tokio::spawn(async move {
                let result = media::process_photo(media_data).await.unwrap();
                let mut query = PhotoUpdateQuery::default();
                query.jpg_small = Some(result.jpg_small);
                query.jpg_medium = Some(result.jpg_medium);
                query.jpg_large = Some(result.jpg_large);
                query.jpg_orig = Some(result.jpg_orig);
                query.processing = Some(false);
                query.profile_picture = Some(true);
                Photo::update(&state.db, id, query).await.unwrap();
            });
            encode_media_id(MediaType::ProfilePicture, id)
        }
        "banner" => {
            let id = Photo::insert(&state.db, claims.user_id).await.unwrap();
            tokio::spawn(async move {
                let result = media::process_photo(media_data).await.unwrap();
                let mut query = PhotoUpdateQuery::default();
                query.jpg_small = Some(result.jpg_small);
                query.jpg_medium = Some(result.jpg_medium);
                query.jpg_large = Some(result.jpg_large);
                query.jpg_orig = Some(result.jpg_orig);
                query.processing = Some(false);
                query.banner = Some(true);
                Photo::update(&state.db, id, query).await.unwrap();
            });
            encode_media_id(MediaType::Banner, id)
        }
        _ => return Err((StatusCode::BAD_REQUEST, "cannot find media type").into()),
    };

    Ok(Json(MediaResponse {
        id,
        processing: true,
    }))
}

async fn media_check(
    State(state): State<Arc<SharedState>>,
    Path(id): Path<String>,
) -> axum::response::Result<impl IntoResponse> {
    let (media_type, num_id) = parse_media_id(&id)?;
    Ok(Json(match media_type {
        MediaType::Photo => {
            let photo = Photo::find(&state.db, num_id)
                .await
                .map_err(|_| (StatusCode::NOT_FOUND, MEDIA_NOT_FOUND))?;
            MediaResponse {
                id,
                processing: photo.processing,
            }
        }
        MediaType::Video => {
            let video = Video::find(&state.db, num_id)
                .await
                .map_err(|_| (StatusCode::NOT_FOUND, MEDIA_NOT_FOUND))?;
            MediaResponse {
                id,
                processing: video.processing,
            }
        }
        MediaType::Audio => {
            let audio = Audio::find(&state.db, num_id)
                .await
                .map_err(|_| (StatusCode::NOT_FOUND, MEDIA_NOT_FOUND))?;
            MediaResponse {
                id,
                processing: audio.processing,
            }
        }
        MediaType::ProfilePicture => {
            let photo = Photo::find(&state.db, num_id)
                .await
                .map_err(|_| (StatusCode::NOT_FOUND, MEDIA_NOT_FOUND))?;
            if !photo.profile_picture {
                return Err((StatusCode::BAD_REQUEST, CANNOT_USE_THIS_MEDIA_TYPE).into());
            }
            MediaResponse {
                id,
                processing: photo.processing,
            }
        }
        MediaType::Banner => {
            let photo = Photo::find(&state.db, num_id)
                .await
                .map_err(|_| (StatusCode::NOT_FOUND, MEDIA_NOT_FOUND))?;
            if !photo.banner {
                return Err((StatusCode::BAD_REQUEST, CANNOT_USE_THIS_MEDIA_TYPE).into());
            }
            MediaResponse {
                id,
                processing: photo.processing,
            }
        }
    }))
}

pub fn routes() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/upload", post(media_upload))
        .route("/check/{id}", get(media_check))
        .route("/{id}", get(media_handler))
}
