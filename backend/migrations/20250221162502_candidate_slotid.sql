ALTER TABLE people.candidates
ADD slot_id UUID NOT NULL REFERENCES records.slots(id) on DELETE CASCADE,
DROP COLUMN am_only,
DROP COLUMN pm_only;