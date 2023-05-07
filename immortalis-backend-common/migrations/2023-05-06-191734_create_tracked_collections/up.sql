CREATE TABLE tracked_collections (
    id int not null primary key generated always as identity,
    url varchar NOT NULL UNIQUE,
    tracking_started_at timestamp without time zone NOT NULL DEFAULT now(),
    last_checked timestamp without time zone
);