CREATE TYPE video_status AS ENUM ('archived', 'scheduled_for_archival', 'being_archived', 'archivation_failed'); -- these may not start with Uppercase

ALTER TABLE videos
ADD COLUMN status video_status NOT NULL DEFAULT 'archived';

ALTER TABLE videos
ALTER COLUMN status
DROP DEFAULT;