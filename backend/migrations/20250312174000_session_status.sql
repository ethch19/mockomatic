ALTER TABLE records.sessions
ADD uploaded bool NOT NULL DEFAULT FALSE,
ADD allocated bool NOT NULL DEFAULT FALSE;