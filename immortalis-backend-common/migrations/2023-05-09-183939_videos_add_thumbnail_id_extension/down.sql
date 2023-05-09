-- This file should undo anything in `up.sql`
ALTER TABLE videos
DROP COLUMN thumbnail_id,
DROP COLUMN thumbnail_extension;