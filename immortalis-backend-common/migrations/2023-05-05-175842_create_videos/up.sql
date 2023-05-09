CREATE TYPE video_status AS ENUM ('archived', 'scheduled_for_archival', 'being_archived', 'archivation_failed'); -- these may not start with Uppercase

CREATE TABLE videos (
  id int GENERATED ALWAYS AS IDENTITY,
  title VARCHAR NOT NULL,
  channel varchar NOT NULL,
  views bigint NOT NULL,
  upload_date timestamp without time zone NOT NULL,
  archived_date timestamp without time zone NOT NULL,
  duration int NOT NULL,
  thumbnail_address varchar NOT NULL,
  original_url varchar NOT NULL UNIQUE,
  status video_status NOT NULL,
  file_id UUID NOT NULL,
  file_extension VARCHAR NOT NULL,
  thumbnail_id UUID NOT NULL,
  thumbnail_extension VARCHAR NOT NULL,
  PRIMARY KEY(id)
);

CREATE TABLE downloads (
    id int GENERATED ALWAYS AS IDENTITY,
    video_id int NOT NULL,
    title VARCHAR NOT NULL,
    value VARCHAR NOT NULL,
    PRIMARY KEY(id),
    CONSTRAINT fk_video
        foreign key (video_id)
            references videos(id)
);


CREATE TABLE scheduled_archivals (
    id int not null primary key generated always as identity,
    url varchar NOT NULL,
    scheduled_at timestamp without time zone NOT NULL DEFAULT now(),
    not_before timestamp without time zone NOT NULL DEFAULT now(),
    CONSTRAINT unique_url UNIQUE (url)
);


CREATE TABLE tracked_collections (
    id int not null primary key generated always as identity,
    url varchar NOT NULL UNIQUE,
    tracking_started_at timestamp without time zone NOT NULL DEFAULT now(),
    last_checked timestamp without time zone
);