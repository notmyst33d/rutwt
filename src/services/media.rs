use std::{
    path::PathBuf,
    process::{Output, Stdio},
};
use tokio::{fs, process::Command};

pub struct PhotoResult {
    pub jpg_small: Vec<u8>,
    pub jpg_medium: Option<Vec<u8>>,
    pub jpg_large: Option<Vec<u8>>,
}

pub struct VideoResult {
    pub thumbnail: Vec<u8>,
    pub mp4_480p: Vec<u8>,
}

pub struct AudioResult {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub thumbnail: Option<Vec<u8>>,
    pub mp3_128k: Vec<u8>,
}

#[derive(serde::Deserialize)]
pub struct FfprobeResult {
    pub streams: Vec<FfprobeStream>,
    pub format: FfprobeFormat,
}

#[derive(serde::Deserialize)]
pub struct FfprobeFormat {
    pub format_name: String,
    pub nb_streams: u64,
    pub tags: Option<FfprobeTags>,
}

#[derive(serde::Deserialize, Clone)]
pub struct FfprobeTags {
    #[serde(alias = "Title", alias = "TITLE")]
    pub title: Option<String>,
    #[serde(alias = "Artist", alias = "ARTIST")]
    pub artist: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct FfprobeStream {
    pub codec_name: String,
    pub width: Option<u64>,
    pub height: Option<u64>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct MediaError {
    pub error: String,
    pub ffmpeg_error: Option<String>,
}

impl MediaError {
    pub fn new(error: String, ffmpeg_error: Option<String>) -> Self {
        Self {
            error,
            ffmpeg_error,
        }
    }
}

impl From<&str> for MediaError {
    fn from(value: &str) -> Self {
        Self {
            error: value.to_string(),
            ffmpeg_error: None,
        }
    }
}

async fn ffprobe(input_path: &PathBuf) -> Result<FfprobeResult, MediaError> {
    let p = Command::new("ffprobe")
        .args(&[
            "-v",
            "error",
            "-output_format",
            "json",
            "-show_streams",
            "-show_format",
            input_path.to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| MediaError::from("cannot create ffprobe process"))?;

    let out = p
        .wait_with_output()
        .await
        .map_err(|_| MediaError::from("cannot create ffprobe process"))?;

    serde_json::from_str::<FfprobeResult>(
        &String::from_utf8(out.stdout)
            .map_err(|_| MediaError::from("cannot parse ffprobe data"))?,
    )
    .map_err(|e| MediaError::from(e.to_string().as_str()))
}

async fn ffmpeg(
    input_path: &PathBuf,
    output_path: &PathBuf,
    input_args: &[&str],
) -> Result<Output, MediaError> {
    let args = [
        &["-i", input_path.to_str().unwrap(), "-map_metadata", "-1"],
        input_args,
        &[output_path.to_str().unwrap()],
    ]
    .concat();

    let p = Command::new("ffmpeg")
        .args(&args)
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| MediaError::from("cannot create ffmpeg process"))?;

    p.wait_with_output()
        .await
        .map_err(|_| MediaError::from("cannot create ffmpeg process"))
}

pub async fn process_photo(input: Vec<u8>) -> Result<PhotoResult, MediaError> {
    let temp_dir = tempfile::tempdir().map_err(|_| MediaError::from("cannot create temp dir"))?;
    let input_path_bin = temp_dir.path().join("input.bin");

    fs::write(&input_path_bin, input)
        .await
        .map_err(|_| MediaError::from("cannot write to input file"))?;

    let probe_result = ffprobe(&input_path_bin).await?;
    if probe_result.format.nb_streams == 0 || probe_result.format.nb_streams > 1 {
        return Err(MediaError::from("invalid stream count"));
    }

    let stream = &probe_result.streams[0];
    let input_path = match stream.codec_name.as_str() {
        "mjpeg" => temp_dir.path().join("input.jpg"),
        "png" => temp_dir.path().join("input.png"),
        "webp" => temp_dir.path().join("input.webp"),
        _ => return Err(MediaError::from("unsupported codec")),
    };

    fs::rename(input_path_bin, &input_path)
        .await
        .map_err(|_| MediaError::from("cannot write to input file"))?;

    let output_small_path = temp_dir.path().join("output_small.jpg");
    let output_medium_path = temp_dir.path().join("output_medium.jpg");
    let output_large_path = temp_dir.path().join("output_large.jpg");

    let output = ffmpeg(&input_path, &output_small_path, &[
        "-vf",
        "scale='min(512,iw)':'min(512,ih)':force_original_aspect_ratio=decrease",
    ])
    .await?;
    if !output.status.success() {
        return Err(MediaError::new(
            "cannot process small photo".into(),
            Some(String::from_utf8(output.stderr).unwrap()),
        ));
    }

    if stream.width.unwrap() > 768 || stream.height.unwrap() > 768 {
        let output = ffmpeg(&input_path, &output_medium_path, &[
            "-vf",
            "scale='min(max(768,iw), 1024)':'min(max(768,ih), 1024)':force_original_aspect_ratio=decrease",
        ])
        .await?;
        if !output.status.success() {
            return Err(MediaError::new(
                "cannot process medium photo".into(),
                Some(String::from_utf8(output.stderr).unwrap()),
            ));
        }
    }

    if stream.width.unwrap() > 2048 || stream.height.unwrap() > 2048 {
        let output = ffmpeg(&input_path, &output_large_path, &[
            "-vf",
            "scale='min(2048,iw)':'min(2048,ih)':force_original_aspect_ratio=decrease",
        ])
        .await?;
        if !output.status.success() {
            return Err(MediaError::new(
                "cannot process large photo".into(),
                Some(String::from_utf8(output.stderr).unwrap()),
            ));
        }
    }

    Ok(PhotoResult {
        jpg_small: fs::read(output_small_path).await.unwrap(),
        jpg_medium: fs::read(output_medium_path).await.ok(),
        jpg_large: fs::read(output_large_path).await.ok(),
    })
}

pub async fn process_video(input: Vec<u8>) -> Result<VideoResult, MediaError> {
    let temp_dir = tempfile::tempdir().map_err(|_| MediaError::from("cannot create temp dir"))?;
    let input_path_bin = temp_dir.path().join("input.bin");

    fs::write(&input_path_bin, input)
        .await
        .map_err(|_| MediaError::from("cannot write to input file"))?;

    let probe_result = ffprobe(&input_path_bin).await?;
    if probe_result.format.nb_streams == 0 {
        return Err(MediaError::from("invalid stream count"));
    }

    let input_path = match probe_result.format.format_name.as_str() {
        "mov,mp4,m4a,3gp,3g2,mj2" => temp_dir.path().join("input.mp4"),
        "matroska,webm" => temp_dir.path().join("input.webm"),
        _ => return Err(MediaError::from("unsupported codec")),
    };

    fs::rename(input_path_bin, &input_path)
        .await
        .map_err(|_| MediaError::from("cannot write to input file"))?;

    let output_thumbnail_path = temp_dir.path().join("output.jpg");
    let output_mp4_480p_path = temp_dir.path().join("output.mp4");

    let output = ffmpeg(&input_path, &output_thumbnail_path, &[
        "-update",
        "true",
        "-frames:v",
        "1",
        "-vf",
        "scale='min(854,iw)':'min(854,ih)':force_original_aspect_ratio=decrease",
    ])
    .await?;
    if !output.status.success() {
        return Err(MediaError::new(
            "cannot process thumbnail photo".into(),
            Some(String::from_utf8(output.stderr).unwrap()),
        ));
    }

    let output = ffmpeg(&input_path, &output_mp4_480p_path, &[
        "-map",
        "0:a:0",
        "-map",
        "0:v:0",
        "-pix_fmt",
        "yuv420p",
        "-crf",
        "28",
        "-preset",
        "ultrafast",
        "-vf",
        "scale='min(854,iw)':'min(854,ih)':force_original_aspect_ratio=decrease",
    ])
    .await?;
    if !output.status.success() {
        return Err(MediaError::new(
            "cannot process 480p video".into(),
            Some(String::from_utf8(output.stderr).unwrap()),
        ));
    }

    Ok(VideoResult {
        thumbnail: fs::read(output_thumbnail_path).await.unwrap(),
        mp4_480p: fs::read(output_mp4_480p_path).await.unwrap(),
    })
}

pub async fn process_audio(input: Vec<u8>) -> Result<AudioResult, MediaError> {
    let temp_dir = tempfile::tempdir().map_err(|_| MediaError::from("cannot create temp dir"))?;
    let input_path_bin = temp_dir.path().join("input.bin");

    fs::write(&input_path_bin, input)
        .await
        .map_err(|_| MediaError::from("cannot write to input file"))?;

    let probe_result = ffprobe(&input_path_bin).await?;
    if probe_result.format.nb_streams == 0 {
        return Err(MediaError::from("invalid stream count"));
    }

    let input_path = match probe_result.format.format_name.as_str() {
        "mp3" => temp_dir.path().join("input.mp3"),
        "flac" => temp_dir.path().join("input.flac"),
        "ogg" => temp_dir.path().join("input.ogg"),
        "mov,mp4,m4a,3gp,3g2,mj2" => temp_dir.path().join("input.m4a"),
        _ => return Err(MediaError::from("unsupported codec")),
    };

    fs::rename(input_path_bin, &input_path)
        .await
        .map_err(|_| MediaError::from("cannot write to input file"))?;

    let output_mp3_128k_path = temp_dir.path().join("output.mp3");
    let output_thumbnail_path = temp_dir.path().join("output.jpg");

    let output = ffmpeg(&input_path, &output_mp3_128k_path, &[
        "-vn", "-map", "0:0", "-b:a", "128k",
    ])
    .await?;
    if !output.status.success() {
        return Err(MediaError::new(
            "cannot process 128k audio".into(),
            Some(String::from_utf8(output.stderr).unwrap()),
        ));
    }

    if probe_result.format.nb_streams == 2 {
        let output = ffmpeg(&input_path, &output_thumbnail_path, &[
            "-an",
            "-map",
            "0:1",
            "-vf",
            "scale=512:512",
        ])
        .await?;
        if !output.status.success() {
            return Err(MediaError::new(
                "cannot process thumbnail audio".into(),
                Some(String::from_utf8(output.stderr).unwrap()),
            ));
        }
    }

    Ok(AudioResult {
        title: probe_result.format.tags.clone().and_then(|v| v.title),
        artist: probe_result.format.tags.and_then(|v| v.artist),
        thumbnail: fs::read(output_thumbnail_path).await.ok(),
        mp3_128k: fs::read(output_mp3_128k_path).await.unwrap(),
    })
}

#[cfg(test)]
mod test {
    use tokio::fs;

    use crate::services::media;

    #[tokio::test]
    async fn photo() {
        let result = media::process_photo(fs::read("testdata/input.png").await.unwrap())
            .await
            .unwrap();
        assert!(result.jpg_small.len() != 0);
        assert!(
            result
                .jpg_medium
                .and_then(|r| Some(r.len() != 0))
                .unwrap_or(true)
        );
        assert!(
            result
                .jpg_large
                .and_then(|r| Some(r.len() != 0))
                .unwrap_or(true)
        );
    }

    #[tokio::test]
    async fn video() {
        let result = media::process_video(fs::read("testdata/input.mp4").await.unwrap())
            .await
            .unwrap();
        assert!(result.thumbnail.len() != 0);
        assert!(result.mp4_480p.len() != 0);
    }

    #[tokio::test]
    async fn audio() {
        let result = media::process_audio(fs::read("testdata/input.mp3").await.unwrap())
            .await
            .unwrap();
        assert!(result.title.unwrap() == "Мой байк");
        assert!(result.artist.unwrap() == "Серега Пират");
        assert!(result.mp3_128k.len() != 0);
        assert!(result.thumbnail.unwrap().len() != 0);
    }
}
