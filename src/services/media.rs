use std::error::Error;

pub struct PhotoResult {
    pub jpg_small: Vec<u8>,
    pub jpg_medium: Vec<u8>,
    pub jpg_large: Vec<u8>,
    pub jpg_orig: Vec<u8>,
}

pub struct VideoResult {
    pub mp4_480p: Vec<u8>,
    pub mp4_720p: Vec<u8>,
}

pub struct AudioResult {
    pub mp3_128k: Vec<u8>,
    pub mp3_320k: Vec<u8>,
}

macro_rules! ffmpeg {
    ($input:expr, $ext:literal) => {{
        use std::process::Stdio;
        use tokio::{fs, process::Command};
        let temp_dir = tempfile::tempdir()?;
        let input_pathbuf = temp_dir.path().join(concat!("input.", $ext));
        let output_pathbuf = temp_dir.path().join(concat!("output.", $ext));
        let input_path = input_pathbuf.to_str().unwrap();
        let output_path = output_pathbuf.to_str().unwrap();

        fs::write(input_path, &$input).await?;

        let p = Command::new("ffmpeg")
            .args(&["-i", input_path, "-map_metadata", "-1", output_path])
            .stderr(Stdio::piped())
            .spawn()?;

        let out = p.wait_with_output().await?;
        let data = fs::read(output_path).await?;
        temp_dir.close()?;
        (out, data)
    }};
    ($input:expr, $ext:literal, $($args:tt),*) => {{
        use std::process::Stdio;
        use tokio::{fs, process::Command};
        let temp_dir = tempfile::tempdir()?;
        let input_pathbuf = temp_dir.path().join(concat!("input.", $ext));
        let output_pathbuf = temp_dir.path().join(concat!("output.", $ext));
        let input_path = input_pathbuf.to_str().unwrap();
        let output_path = output_pathbuf.to_str().unwrap();

        fs::write(input_path, &$input).await?;

        let p = Command::new("ffmpeg")
            .args(&["-i", input_path, "-map_metadata", "-1", $($args),*, output_path])
            .stderr(Stdio::piped())
            .spawn()?;

        let out = p.wait_with_output().await?;
        let data = fs::read(output_path).await?;
        temp_dir.close()?;
        (out, data)
    }};
}

pub async fn process_photo(input: Vec<u8>) -> Result<PhotoResult, Box<dyn Error>> {
    let output_small = ffmpeg!(
        input,
        "jpg",
        "-vf",
        "scale=512:512:force_original_aspect_ratio=decrease",
        "-f",
        "mjpeg"
    );
    if !output_small.0.status.success() {
        return Err(String::from_utf8(output_small.0.stderr)?.into());
    }

    let output_medium = ffmpeg!(
        input,
        "jpg",
        "-vf",
        "scale=1024:1024:force_original_aspect_ratio=decrease",
        "-f",
        "mjpeg"
    );
    if !output_medium.0.status.success() {
        return Err(String::from_utf8(output_medium.0.stderr)?.into());
    }

    let output_large = ffmpeg!(
        input,
        "jpg",
        "-vf",
        "scale=2048:2048:force_original_aspect_ratio=decrease",
        "-f",
        "mjpeg"
    );
    if !output_large.0.status.success() {
        return Err(String::from_utf8(output_large.0.stderr)?.into());
    }

    let output_orig = ffmpeg!(input, "jpg");
    if !output_orig.0.status.success() {
        return Err(String::from_utf8(output_orig.0.stderr)?.into());
    }

    Ok(PhotoResult {
        jpg_small: output_small.1,
        jpg_medium: output_medium.1,
        jpg_large: output_large.1,
        jpg_orig: output_orig.1,
    })
}

pub async fn process_video(input: Vec<u8>) -> Result<VideoResult, Box<dyn Error>> {
    let output_480p = ffmpeg!(
        input,
        "mp4",
        "-pix_fmt",
        "yuv420p",
        "-crf",
        "28",
        "-preset",
        "ultrafast",
        "-vf",
        "scale=854:854:force_original_aspect_ratio=decrease"
    );
    if !output_480p.0.status.success() {
        return Err(String::from_utf8(output_480p.0.stderr)?.into());
    }

    let output_720p = ffmpeg!(
        input,
        "mp4",
        "-pix_fmt",
        "yuv420p",
        "-crf",
        "28",
        "-preset",
        "ultrafast",
        "-vf",
        "scale=1280:1280:force_original_aspect_ratio=decrease"
    );
    if !output_720p.0.status.success() {
        return Err(String::from_utf8(output_720p.0.stderr)?.into());
    }

    Ok(VideoResult {
        mp4_480p: output_480p.1,
        mp4_720p: output_720p.1,
    })
}

pub async fn process_audio(input: Vec<u8>) -> Result<AudioResult, Box<dyn Error>> {
    let output_128k = ffmpeg!(
        input, "mp3", "-vn", "-map", "0:0", "-b:a", "128k", "-f", "mp3"
    );
    if !output_128k.0.status.success() {
        return Err(String::from_utf8(output_128k.0.stderr)?.into());
    }

    let output_320k = ffmpeg!(
        input, "mp3", "-vn", "-map", "0:0", "-b:a", "320k", "-f", "mp3"
    );
    if !output_320k.0.status.success() {
        return Err(String::from_utf8(output_320k.0.stderr)?.into());
    }

    Ok(AudioResult {
        mp3_128k: output_128k.1,
        mp3_320k: output_320k.1,
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
        assert!(result.jpg_medium.len() != 0);
        assert!(result.jpg_large.len() != 0);
        assert!(result.jpg_orig.len() != 0);
    }

    #[tokio::test]
    async fn video() {
        let result = media::process_video(fs::read("testdata/input.mp4").await.unwrap())
            .await
            .unwrap();
        assert!(result.mp4_720p.len() != 0);
    }

    #[tokio::test]
    async fn audio() {
        let result = media::process_audio(fs::read("testdata/input.mp3").await.unwrap())
            .await
            .unwrap();
        assert!(result.mp3_320k.len() != 0);
    }
}
