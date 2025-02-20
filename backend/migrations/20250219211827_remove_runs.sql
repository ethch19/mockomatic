DROP TABLE records.runs;

ALTER TABLE records.circuits
ADD scheduled_start timestamp with time zone NOT NULL,
ADD scheduled_end timestamp with time zone NOT NULL;