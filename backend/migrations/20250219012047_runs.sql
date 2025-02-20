CREATE TABLE IF NOT EXISTS records.runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slot_id UUID NOT NULL REFERENCES records.slots(id) ON DELETE CASCADE,
    scheduled_start timestamp with time zone NOT NULL,
    scheduled_end timestamp with time zone NOT NULL,
    timer_start timestamp with time zone,
    timer_end timestamp with time zone
);

ALTER TABLE records.slots
DROP COLUMN scheduled_start,
DROP COLUMN scheduled_end;