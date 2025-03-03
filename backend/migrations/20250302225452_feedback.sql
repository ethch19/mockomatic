ALTER TABLE records.sessions
ADD feedback bool NOT NULL,
ADD feedback_duration interval;

ALTER TABLE records.circuits
ADD feedback bool NOT NULL;

ALTER TABLE templates.sessions
ADD feedback bool NOT NULL,
ADD feedback_duration interval;