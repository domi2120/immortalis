-- Your SQL goes here
ALTER TABLE videos
ADD COLUMN file_id UUID NOT NULL,
ADD CONSTRAINT unique_file_id UNIQUE(file_id)
