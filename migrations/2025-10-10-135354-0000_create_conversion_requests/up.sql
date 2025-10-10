-- Create conversion_requests table for security classification conversion requests
CREATE TABLE IF NOT EXISTS conversion_requests (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    creator_id UUID NOT NULL,
        FOREIGN KEY(creator_id)
        REFERENCES users(id) ON DELETE RESTRICT,
    authority_id UUID NOT NULL,
        FOREIGN KEY(authority_id)
        REFERENCES authorities(id) ON DELETE RESTRICT,
    data_object_id UUID NOT NULL,
        FOREIGN KEY(data_object_id)
        REFERENCES data_objects(id) ON DELETE CASCADE,
    source_nation_code VARCHAR(3) NOT NULL,
    target_nation_codes TEXT[] NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP DEFAULT NULL
);

-- Indexes for efficient querying
CREATE INDEX conversion_requests__creator_id_idx ON conversion_requests(creator_id);
CREATE INDEX conversion_requests__authority_id_idx ON conversion_requests(authority_id);
CREATE INDEX conversion_requests__data_object_id_idx ON conversion_requests(data_object_id);
CREATE INDEX conversion_requests__source_nation_code_idx ON conversion_requests(source_nation_code);
CREATE INDEX conversion_requests__created_at_idx ON conversion_requests(created_at);
CREATE INDEX conversion_requests__completed_at_idx ON conversion_requests(completed_at);
