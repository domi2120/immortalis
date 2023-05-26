CREATE TYPE video_status AS ENUM ('archived', 'scheduled_for_archival', 'being_archived', 'archivation_failed'); -- these may not start with Uppercase

CREATE TABLE files (
  id UUID not null primary key DEFAULT gen_random_uuid(),
  file_name varchar NOT NULL,
  file_extension varchar NOT NULL,
  size bigint NOT NULL
);

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
  thumbnail_id UUID NOT NULL,
  PRIMARY KEY(id),
    CONSTRAINT fk_file_video
        foreign key (file_id)
            references files(id),
    CONSTRAINT fk_file_thumbnail
        foreign key (thumbnail_id)
            references files(id)
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

CREATE OR REPLACE FUNCTION notify_delete_insert()
RETURNS trigger AS
$$
DECLARE
  payload TEXT;
  rec RECORD;
BEGIN
	if tg_op = 'INSERT' OR TG_OP = 'UPDATE' then
      rec := NEW;
	elsif tg_op = 'DELETE' then
      REC := OLD;
	end if;

  payload := json_build_object('action',LOWER(TG_OP), 'record',row_to_json(rec));
  PERFORM pg_notify(TG_TABLE_NAME, payload);
  RETURN NEW;
END;
$$ LANGUAGE 'plpgsql';

CREATE OR REPLACE TRIGGER scheduled_archivals_after_delete_insert_trigger AFTER DELETE OR INSERT
       ON scheduled_archivals
       FOR EACH ROW EXECUTE PROCEDURE notify_delete_insert();


CREATE OR REPLACE TRIGGER scheduled_archivals_after_delete_insert_trigger AFTER DELETE OR INSERT OR UPDATE
       ON tracked_collections
       FOR EACH ROW EXECUTE PROCEDURE notify_delete_insert();