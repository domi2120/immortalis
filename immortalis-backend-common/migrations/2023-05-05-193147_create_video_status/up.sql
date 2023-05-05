CREATE TYPE video_status AS ENUM ('archived', 'scheduledForArchival', 'beingArchived', 'archivationFailed'); -- these may not start with Uppercase

ALTER TABLE videos
ADD COLUMN status video_status NOT NULL DEFAULT 'archived';

ALTER TABLE videos
ALTER COLUMN status
DROP DEFAULT;