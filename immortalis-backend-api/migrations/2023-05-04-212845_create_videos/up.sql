CREATE TABLE videos (
  id int GENERATED ALWAYS AS IDENTITY,
  title VARCHAR NOT NULL,
  channel varchar NOT NULL,
  views bigint NOT NULL,
  upload_date timestamp without time zone NOT NULL,
  archived_date timestamp without time zone NOT NULL,
  duration int NOT NULL,
  thumbnail_address varchar NOT NULL,
  original_url varchar NOT NULL,
  PRIMARY KEY(id)
);

CREATE TABLE videoDownloads (
    id int GENERATED ALWAYS AS IDENTITY,
    video_id int NOT NULL,
    title VARCHAR NOT NULL,
    value VARCHAR NOT NULL,
    PRIMARY KEY(id),
    CONSTRAINT fk_video
        foreign key (video_id)
            references videos(id)
)