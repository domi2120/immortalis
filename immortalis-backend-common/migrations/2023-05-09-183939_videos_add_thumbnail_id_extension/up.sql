-- Your SQL goes here
ALTER TABLE videos
ADD COLUMN thumbnail_id UUID NOT NULL,
ADD COLUMN thumbnail_extension VARCHAR NOT NULL;