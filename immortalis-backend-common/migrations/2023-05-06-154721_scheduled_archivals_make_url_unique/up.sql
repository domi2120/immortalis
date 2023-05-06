ALTER TABLE scheduled_archivals
ADD CONSTRAINT unique_url UNIQUE (url)
