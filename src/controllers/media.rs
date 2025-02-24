use std::{sync::Arc, time::Duration};

use axum::{
    Json, Router,
    extract::{Multipart, Path, State},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
    routing::{get, head, post},
};
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use tokio::time::sleep;

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
    pub processing_error: Option<String>,
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

async fn media_handler_head() -> impl IntoResponse {
    [(header::ACCEPT_RANGES, "bytes")]
}

#[derive(serde::Serialize)]
struct MediaMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub thumbnail: bool,
}

async fn media_metadata(
    State(state): State<Arc<SharedState>>,
    Path(id): Path<String>,
) -> axum::response::Result<impl IntoResponse> {
    let (media_type, num_id) = parse_media_id(&id)?;
    if media_type != MediaType::Audio {
        return Err((StatusCode::BAD_REQUEST, CANNOT_USE_THIS_MEDIA_TYPE).into());
    }

    let audio = Audio::find(&state.db, num_id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, MEDIA_NOT_FOUND))?;

    Ok((
        StatusCode::OK,
        Json(MediaMetadata {
            title: audio.title,
            artist: audio.artist,
            thumbnail: audio.thumbnail.is_some(),
        }),
    ))
}

async fn media_handler(
    request_headers: HeaderMap,
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

    let default_res = match media_type {
        MediaType::Audio => "128k",
        MediaType::Photo | MediaType::Banner => "medium",
        MediaType::ProfilePicture => "small",
        MediaType::Video => "480p",
    };
    let (_, preferred_res) = ext.split_once(":").unwrap_or((ext, default_res));

    let (media, filename) = match &ext[..3] {
        "jpg" => {
            headers.insert(header::CONTENT_TYPE, "image/jpeg".parse().unwrap());
            let (media, actual_res) = match media_type {
                MediaType::Audio => {
                    let audio = Audio::find(&state.db, num_id)
                        .await
                        .map_err(|_| (StatusCode::NOT_FOUND, MEDIA_NOT_FOUND))?;
                    if audio.processing || audio.thumbnail == None {
                        return Err((StatusCode::NO_CONTENT, MEDIA_IS_PROCESSING).into());
                    }
                    (audio.thumbnail, "thumbnail")
                }
                MediaType::Video => {
                    let video = Video::find(&state.db, num_id)
                        .await
                        .map_err(|_| (StatusCode::NOT_FOUND, MEDIA_NOT_FOUND))?;
                    if video.thumbnail == None {
                        return Err((StatusCode::NOT_FOUND, MEDIA_NOT_FOUND).into());
                    }
                    if video.processing {
                        return Err((StatusCode::NO_CONTENT, MEDIA_IS_PROCESSING).into());
                    }
                    (video.thumbnail, "thumbnail")
                }
                MediaType::Photo | MediaType::Banner | MediaType::ProfilePicture => {
                    let photo = Photo::find(&state.db, num_id)
                        .await
                        .map_err(|_| (StatusCode::NOT_FOUND, MEDIA_NOT_FOUND))?;
                    if photo.processing {
                        return Err((StatusCode::NO_CONTENT, MEDIA_IS_PROCESSING).into());
                    }
                    if (!photo.profile_picture && media_type == MediaType::ProfilePicture)
                        || (photo.profile_picture && media_type == MediaType::Photo)
                        || (photo.profile_picture && media_type == MediaType::Banner)
                        || (!photo.banner && media_type == MediaType::Banner)
                        || (photo.banner && media_type == MediaType::Photo)
                        || (photo.banner && media_type == MediaType::ProfilePicture)
                    {
                        return Err((StatusCode::BAD_REQUEST, CANNOT_USE_THIS_MEDIA_TYPE).into());
                    }
                    let formats = [
                        (photo.jpg_large, "large"),
                        (photo.jpg_medium, "medium"),
                        (photo.jpg_small, "small"),
                    ];
                    let mut best_format = formats
                        .clone()
                        .into_iter()
                        .find(|f| f.0.is_some() && f.1 == preferred_res);
                    if best_format.is_none() {
                        best_format = formats.into_iter().find(|f| f.0.is_some());
                    }
                    if best_format.is_none() {
                        return Err((StatusCode::NO_CONTENT, MEDIA_IS_PROCESSING).into());
                    }
                    best_format.unwrap()
                }
            };
            (media.unwrap(), format!("{id}_{actual_res}.jpg"))
        }
        "mp4" => {
            if media_type != MediaType::Video {
                return Err((StatusCode::BAD_REQUEST, CANNOT_USE_THIS_MEDIA_TYPE).into());
            }
            headers.insert(header::CONTENT_TYPE, "video/mp4".parse().unwrap());
            let video = Video::find(&state.db, num_id)
                .await
                .map_err(|_| (StatusCode::NOT_FOUND, MEDIA_NOT_FOUND))?;
            if video.processing {
                return Err((StatusCode::NO_CONTENT, MEDIA_IS_PROCESSING).into());
            }
            let formats = [(video.mp4_480p, "480p")];
            let mut best_format = formats
                .clone()
                .into_iter()
                .find(|f| f.0.is_some() && f.1 == preferred_res);
            if best_format.is_none() {
                best_format = formats.into_iter().find(|f| f.0.is_some());
            }
            if best_format.is_none() {
                return Err((StatusCode::NO_CONTENT, MEDIA_IS_PROCESSING).into());
            }
            let (media, actual_res) = best_format.unwrap();
            (media.unwrap(), format!("{id}_{actual_res}.jpg"))
        }
        "mp3" => {
            if media_type != MediaType::Audio {
                return Err((StatusCode::BAD_REQUEST, CANNOT_USE_THIS_MEDIA_TYPE).into());
            }
            headers.insert(header::CONTENT_TYPE, "audio/mpeg".parse().unwrap());
            let audio = Audio::find(&state.db, num_id)
                .await
                .map_err(|_| (StatusCode::NOT_FOUND, MEDIA_NOT_FOUND))?;
            if audio.processing {
                return Err((StatusCode::NO_CONTENT, MEDIA_IS_PROCESSING).into());
            }
            let formats = [(audio.mp3_128k, "128k")];
            let mut best_format = formats
                .clone()
                .into_iter()
                .find(|f| f.0.is_some() && f.1 == preferred_res);
            if best_format.is_none() {
                best_format = formats.into_iter().find(|f| f.0.is_some());
            }
            if best_format.is_none() {
                return Err((StatusCode::NO_CONTENT, MEDIA_IS_PROCESSING).into());
            }
            let (media, actual_res) = best_format.unwrap();
            (media.unwrap(), format!("{id}_{actual_res}.mp3"))
        }
        _ => return Err((StatusCode::BAD_REQUEST, CANNOT_USE_THIS_MEDIA_TYPE).into()),
    };

    headers.insert(header::ACCEPT_RANGES, "bytes".parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("inline; filename=\"{filename}\"").parse().unwrap(),
    );

    if let Some(range) = request_headers.get("Range") {
        let range = range.to_str().unwrap();
        if range.contains(",") {
            return Ok((StatusCode::OK, headers, media));
        }
        let Some((from, to)) = range[6..].split_once("-") else {
            return Ok((StatusCode::OK, headers, media));
        };
        let start = match from.parse::<usize>() {
            Ok(v) => v,
            Err(_) => 0,
        };
        let end = match to.parse::<usize>() {
            Ok(v) => v + 1,
            Err(_) => media.len(),
        };
        if start > media.len() || end > media.len() {
            return Err((StatusCode::RANGE_NOT_SATISFIABLE, headers, "").into());
        }
        if start == 0 && end == media.len() {
            return Ok((StatusCode::OK, headers, media));
        } else {
            headers.insert(
                header::CONTENT_RANGE,
                format!("bytes {start}-{}/{}", end - 1, media.len())
                    .parse()
                    .unwrap(),
            );
            return Ok((
                StatusCode::PARTIAL_CONTENT,
                headers,
                media[start..end].to_vec(),
            ));
        }
    }

    Ok((StatusCode::OK, headers, media))
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
        let data = field
            .bytes()
            .await
            .map_err(|_| (StatusCode::BAD_REQUEST, "cannot read body"))?;

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
        "photo" | "profile_picture" | "banner" => {
            let id = Photo::insert(&state.db, claims.user_id).await.unwrap();
            let media_type_id = match media_type.as_str() {
                "profile_picture" => MediaType::ProfilePicture,
                "banner" => MediaType::Banner,
                _ => MediaType::Photo,
            };
            tokio::spawn(async move {
                let mut query = PhotoUpdateQuery::default();
                let result = match media::process_photo(media_data).await {
                    Ok(r) => r,
                    Err(e) => {
                        query.processing = Some(false);
                        query.processing_error = Some(e.error);
                        Photo::update(&state.db, id, query).await.unwrap();
                        sleep(Duration::from_secs(10)).await;
                        Photo::delete(&state.db, id).await.unwrap();
                        return;
                    }
                };
                query.processing = Some(false);
                query.jpg_small = Some(result.jpg_small);
                query.jpg_medium = result.jpg_medium;
                query.jpg_large = result.jpg_large;
                query.profile_picture = Some(media_type == "profile_picture");
                query.banner = Some(media_type == "banner");
                Photo::update(&state.db, id, query).await.unwrap();
            });
            encode_media_id(media_type_id, id)
        }
        "video" => {
            let id = Video::insert(&state.db, claims.user_id).await.unwrap();
            tokio::spawn(async move {
                let mut query = VideoUpdateQuery::default();
                let result = match media::process_video(media_data).await {
                    Ok(r) => r,
                    Err(e) => {
                        query.processing = Some(false);
                        query.processing_error = Some(e.error);
                        Video::update(&state.db, id, query).await.unwrap();
                        sleep(Duration::from_secs(10)).await;
                        Video::delete(&state.db, id).await.unwrap();
                        return;
                    }
                };
                query.processing = Some(false);
                query.thumbnail = Some(result.thumbnail);
                query.mp4_480p = Some(result.mp4_480p);
                Video::update(&state.db, id, query).await.unwrap();
            });
            encode_media_id(MediaType::Video, id)
        }
        "audio" => {
            let id = Audio::insert(&state.db, claims.user_id).await.unwrap();
            tokio::spawn(async move {
                let mut query = AudioUpdateQuery::default();
                let result = match media::process_audio(media_data).await {
                    Ok(r) => r,
                    Err(e) => {
                        query.processing = Some(false);
                        query.processing_error = Some(e.error);
                        Audio::update(&state.db, id, query).await.unwrap();
                        sleep(Duration::from_secs(10)).await;
                        Audio::delete(&state.db, id).await.unwrap();
                        return;
                    }
                };
                query.processing = Some(false);
                query.title = result.title.and_then(|t| {
                    Some(if t.len() > 100 {
                        t[..100].to_string()
                    } else {
                        t
                    })
                });
                query.artist = result.artist.and_then(|a| {
                    Some(if a.len() > 100 {
                        a[..100].to_string()
                    } else {
                        a
                    })
                });
                query.thumbnail = result.thumbnail;
                query.mp3_128k = Some(result.mp3_128k);
                Audio::update(&state.db, id, query).await.unwrap();
            });
            encode_media_id(MediaType::Audio, id)
        }
        _ => return Err((StatusCode::BAD_REQUEST, "cannot find media type").into()),
    };

    Ok(Json(MediaResponse {
        id,
        processing: true,
        processing_error: None,
    }))
}

async fn media_check(
    State(state): State<Arc<SharedState>>,
    Path(id): Path<String>,
) -> axum::response::Result<impl IntoResponse> {
    let (media_type, num_id) = parse_media_id(&id)?;
    Ok(Json(match media_type {
        MediaType::Photo | MediaType::Banner | MediaType::ProfilePicture => {
            let photo = Photo::find(&state.db, num_id)
                .await
                .map_err(|_| (StatusCode::NOT_FOUND, MEDIA_NOT_FOUND))?;
            MediaResponse {
                id,
                processing: photo.processing,
                processing_error: photo.processing_error,
            }
        }
        MediaType::Video => {
            let video = Video::find(&state.db, num_id)
                .await
                .map_err(|_| (StatusCode::NOT_FOUND, MEDIA_NOT_FOUND))?;
            MediaResponse {
                id,
                processing: video.processing,
                processing_error: video.processing_error,
            }
        }
        MediaType::Audio => {
            let audio = Audio::find(&state.db, num_id)
                .await
                .map_err(|_| (StatusCode::NOT_FOUND, MEDIA_NOT_FOUND))?;
            MediaResponse {
                id,
                processing: audio.processing,
                processing_error: audio.processing_error,
            }
        }
    }))
}

pub fn routes() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/upload", post(media_upload))
        .route("/check/{id}", get(media_check))
        .route("/metadata/{id}", get(media_metadata))
        .route("/{id}", get(media_handler))
        .route("/{id}", head(media_handler_head))
}
