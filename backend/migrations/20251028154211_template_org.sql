BEGIN;

ALTER TABLE templates.sessions
ADD COLUMN organisation_id UUID NOT NULL
    REFERENCES auth.organisations(id)
    ON DELETE CASCADE;

ALTER TABLE templates.sessions
DROP CONSTRAINT sessions_name_key;

ALTER TABLE templates.sessions
ADD CONSTRAINT sessions_organisation_id_name_key
  UNIQUE (organisation_id, name);

CREATE INDEX idx_sessions_organisation_id
  ON templates.sessions (organisation_id);

COMMIT;