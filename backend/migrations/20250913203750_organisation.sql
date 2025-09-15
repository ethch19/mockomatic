CREATE TABLE IF NOT EXISTS auth.organisations (
	id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
	name text NOT NULL UNIQUE
);

ALTER TABLE auth.users
ADD COLUMN organisation_id UUID NOT NULL REFERENCES auth.organisations(id) ON DELETE CASCADE;

ALTER TABLE records.sessions
DROP COLUMN organisation,
ADD COLUMN organisation_id UUID NOT NULL REFERENCES auth.organisations(id) ON DELETE CASCADE;

ALTER TABLE records.sessions
DROP CONSTRAINT sessions_organiser_id_fkey;

ALTER TABLE records.sessions
ADD CONSTRAINT sessions_organiser_id_fkey
FOREIGN KEY (organiser_id)
REFERENCES auth.users(id)
ON DELETE CASCADE;