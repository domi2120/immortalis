ALTER TABLE scheduled_archivals
ADD COLUMN not_before timestamp without time zone NOT NULL DEFAULT now();