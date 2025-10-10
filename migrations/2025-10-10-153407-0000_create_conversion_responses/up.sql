CREATE TABLE conversion_responses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversion_request_id UUID NOT NULL REFERENCES conversion_requests(id) ON DELETE CASCADE,
    subject_data_id UUID NOT NULL REFERENCES data_objects(id) ON DELETE CASCADE,
    nato_equivalent VARCHAR(128) NOT NULL,
    target_nation_classifications JSONB NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP
);

-- Index for faster lookups by conversion request
CREATE INDEX idx_conversion_responses_request_id ON conversion_responses(conversion_request_id);

-- Index for faster lookups by subject data
CREATE INDEX idx_conversion_responses_subject_data_id ON conversion_responses(subject_data_id);
