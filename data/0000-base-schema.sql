CREATE TABLE posts (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_id INTEGER NOT NULL,
    message TEXT,
    comment BIT NOT NULL
);

CREATE TABLE likes (
    post_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL
);

CREATE TABLE comments (
    post_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    comment_post_id INTEGER NOT NULL
);

CREATE TABLE reposts (
    post_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL
);

CREATE TABLE posts_photos (
    post_id INTEGER NOT NULL,
    photo_id INTEGER NOT NULL
);

CREATE TABLE posts_videos (
    post_id INTEGER NOT NULL,
    video_id INTEGER NOT NULL
);

CREATE TABLE posts_audios (
    post_id INTEGER NOT NULL,
    audio_id INTEGER NOT NULL
);

CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    username TEXT NOT NULL UNIQUE,
    realname TEXT NOT NULL,
    hashed_password TEXT NOT NULL,
    profile_picture_photo_id INTEGER,
    banner_photo_id INTEGER,
    bio TEXT
);

CREATE TABLE follows (
    user_id INTEGER NOT NULL,
    sub_user_id INTEGER NOT NULL
);

CREATE TABLE photos (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_id INTEGER NOT NULL,
    processing BIT NOT NULL,
    profile_picture BIT NOT NULL,
    banner BIT NOT NULL,
    jpg_small BLOB,
    jpg_medium BLOB,
    jpg_large BLOB,
    jpg_orig BLOB
);

CREATE TABLE videos (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_id INTEGER NOT NULL,
    processing BIT NOT NULL,
    mp4_480p BLOB,
    mp4_720p BLOB
);

CREATE TABLE audios (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_id INTEGER NOT NULL,
    processing BIT NOT NULL,
    mp3_128k BLOB,
    mp3_320k BLOB
);
