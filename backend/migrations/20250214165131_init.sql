CREATE SCHEMA IF NOT EXISTS auth;
CREATE SCHEMA IF NOT EXISTS records;
CREATE SCHEMA IF NOT EXISTS people;
CREATE SCHEMA IF NOT EXISTS templates;

CREATE TABLE IF NOT EXISTS auth.users (
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    username varchar(20) NOT NULL UNIQUE,
    password text NOT NULL,
    admin bool NOT NULL,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS records.sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organiser_id UUID NOT NULL REFERENCES auth.users(id),
    organisation text NOT NULL,
    date date NOT NULL,
    location text NOT NULL,
    total_stations smallint NOT NULL,
    intermission_duration interval NOT NULL,
    static_at_end bool NOT NULL, -- whether to run the irregularly timed station last (only if there is only 1 irregular)
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS records.slots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES records.sessions(id) ON DELETE CASCADE,
    slot_time char(2) NOT NULL, -- stored as "AM" or "PM"
    scheduled_start timestamp with time zone,
    scheduled_end timestamp with time zone
);

CREATE TABLE IF NOT EXISTS records.circuits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES records.sessions(id) ON DELETE CASCADE,
    slot_id UUID NOT NULL REFERENCES records.slots(id) ON DELETE CASCADE,
    key char(1) NOT NULL, -- storing A-Z
    female_only bool NOT NULL,
    current_rotation smallint, -- only stored during "running", else null
    status text NOT NULL DEFAULT 'pending', -- "pending", "running", "completed"
    intermission bool NOT NULL,
    timer_start timestamp with time zone,
    timer_end timestamp with time zone,
    UNIQUE (session_id, key)
);

CREATE TABLE IF NOT EXISTS records.stations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES records.sessions(id) ON DELETE CASCADE,
    title text NOT NULL,
    index smallint NOT NULL,
    duration interval NOT NULL
);

CREATE TABLE IF NOT EXISTS people.candidates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES records.sessions(id) ON DELETE CASCADE,
    first_name text NOT NULL,
    last_name text NOT NULL,
    shortcode text NOT NULL,
    female_only bool NOT NULL, -- whether they chose to be in female-only circuit or not
    partner_pref text, -- partner shortcode
    checked_in bool NOT NULL
);

CREATE TABLE IF NOT EXISTS people.examiners (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES records.sessions(id) ON DELETE CASCADE,
    first_name text NOT NULL,
    last_name text NOT NULL,
    shortcode text NOT NULL,
    female bool NOT NULL,
    AM bool NOT NULL,
    PM bool NOT NULL,
    checked_in bool NOT NULL
);

CREATE TABLE IF NOT EXISTS records.allocations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slot_id UUID NOT NULL REFERENCES records.slots(id) ON DELETE CASCADE,
    circuit_id UUID NOT NULL REFERENCES records.circuits(id) ON DELETE CASCADE,
    station_id UUID NOT NULL REFERENCES records.stations(id) ON DELETE CASCADE,
    candidate_1 UUID NOT NULL REFERENCES people.candidates(id) ON DELETE CASCADE,
    candidate_2 UUID NOT NULL REFERENCES people.candidates(id) ON DELETE CASCADE,
    examiner UUID NOT NULL REFERENCES people.examiners(id) ON DELETE CASCADE,
    modified_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- Ensure a candidate does not appear in candidate_1 or candidate_2 in the same slot & circuit
    CONSTRAINT unique_candidate_pair UNIQUE (slot_id, circuit_id, candidate_1, candidate_2),
    -- Ensure an examiner does not appear more than once in the same slot & circuit
    CONSTRAINT unique_examiner UNIQUE (slot_id, circuit_id, examiner),
    -- Ensure candidate_1 and candidate_2 are not the same person
    CONSTRAINT different_candidates CHECK (candidate_1 <> candidate_2)
);

CREATE TABLE IF NOT EXISTS records.allocations_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    batch_id UUID NOT NULL,
    slot_id UUID NOT NULL REFERENCES records.slots(id) ON DELETE CASCADE,
    circuit_id UUID NOT NULL REFERENCES records.circuits(id) ON DELETE CASCADE,
    station_id UUID NOT NULL REFERENCES records.stations(id) ON DELETE CASCADE,
    candidate_1 UUID NOT NULL REFERENCES people.candidates(id) ON DELETE CASCADE,
    candidate_2 UUID NOT NULL REFERENCES people.candidates(id) ON DELETE CASCADE,
    examiner UUID NOT NULL REFERENCES people.examiners(id) ON DELETE CASCADE,
    modified_by UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    auto_gen bool NOT NULL, -- whether this change is made by algorithm
    modified_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_allocations_history_modified_at
ON records.allocations_history (modified_at DESC);

CREATE TABLE IF NOT EXISTS templates.sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL UNIQUE,
    total_stations smallint NOT NULL,
    intermission_duration interval NOT NULL,
    static_at_end bool NOT NULL -- whether to run the irregularly timed station last (only if there is only 1 irregular)
);

CREATE TABLE IF NOT EXISTS templates.stations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES templates.sessions(id) ON DELETE CASCADE,
    title text NOT NULL,
    index smallint NOT NULL,
    duration interval NOT NULL
);
