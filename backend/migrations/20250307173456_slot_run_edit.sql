ALTER TABLE records.slots
DROP slot_time,
ADD key char(1) NOT NULL;

ALTER TABLE records.runs
ADD flip_allocation bool NOT NULL;