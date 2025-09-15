ALTER TABLE records.sessions
DROP uploaded,
DROP allocated,
ADD status text NOT NULL DEFAULT 'new';