ALTER TABLE people.candidates
ADD CONSTRAINT unique_candidates_session_shortcode
UNIQUE (session_id, shortcode);

ALTER TABLE people.examiners
ADD CONSTRAINT unique_examiners_session_shortcode
UNIQUE (session_id, shortcode);