CREATE TABLE scheduled_archivals (
    id int not null primary key generated always as identity,
    url varchar NOT NULL,
    scheduled_at timestamp without time zone NOT NULL DEFAULT now()
);