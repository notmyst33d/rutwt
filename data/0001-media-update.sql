ALTER TABLE photos ADD processing_error TEXT;
ALTER TABLE photos DROP COLUMN jpg_orig;

ALTER TABLE videos ADD processing_error TEXT;
ALTER TABLE videos ADD thumbnail BLOB;
ALTER TABLE videos DROP COLUMN mp4_720p;

ALTER TABLE audios ADD processing_error TEXT;
ALTER TABLE audios ADD title TEXT;
ALTER TABLE audios ADD artist TEXT;
ALTER TABLE audios ADD thumbnail BLOB;
ALTER TABLE audios DROP COLUMN mp3_320k;

ALTER TABLE posts ADD deleted BIT NOT NULL DEFAULT 0;
ALTER TABLE users ADD deleted BIT NOT NULL DEFAULT 0;
