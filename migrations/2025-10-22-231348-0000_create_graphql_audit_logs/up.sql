-- Create GraphQL Audit Logs table for security and analytics tracking
-- This table captures all GraphQL queries and mutations for data provenance

CREATE TYPE graphql_operation_type AS ENUM (
    'query',
    'mutation',
    'subscription'
);

CREATE TABLE IF NOT EXISTS graphql_audit_logs (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,

    -- User identification and context
    user_id UUID,
        FOREIGN KEY(user_id)
        REFERENCES users(id) ON DELETE SET NULL,
    user_role VARCHAR(64),
    user_access_level VARCHAR(64),

    -- Organization/Authority context (for multi-tenant tracking)
    authority_id UUID,
        FOREIGN KEY(authority_id)
        REFERENCES authorities(id) ON DELETE SET NULL,
    nation_code VARCHAR(3),

    -- GraphQL operation details
    operation_type graphql_operation_type NOT NULL,
    operation_name VARCHAR(256),
    query_text TEXT NOT NULL,
    variables_json JSONB,

    -- Request metadata
    request_id UUID DEFAULT gen_random_uuid(),
    client_ip VARCHAR(45), -- IPv6 max length
    user_agent TEXT,

    -- Response details
    execution_time_ms INTEGER,
    response_status VARCHAR(32) NOT NULL, -- 'success', 'error', 'partial'
    error_message TEXT,
    errors_json JSONB,

    -- Data access tracking (what data was accessed)
    accessed_data_objects UUID[],
    accessed_classifications text[],

    -- Audit timestamps
    executed_at TIMESTAMP NOT NULL DEFAULT NOW(),

    -- Additional context
    session_id VARCHAR(256),
    request_headers JSONB
);

-- Indexes for efficient querying and security analysis
CREATE INDEX graphql_audit_logs__user_id_idx ON graphql_audit_logs(user_id);
CREATE INDEX graphql_audit_logs__authority_id_idx ON graphql_audit_logs(authority_id);
CREATE INDEX graphql_audit_logs__nation_code_idx ON graphql_audit_logs(nation_code);
CREATE INDEX graphql_audit_logs__operation_type_idx ON graphql_audit_logs(operation_type);
CREATE INDEX graphql_audit_logs__operation_name_idx ON graphql_audit_logs(operation_name);
CREATE INDEX graphql_audit_logs__executed_at_idx ON graphql_audit_logs(executed_at);
CREATE INDEX graphql_audit_logs__request_id_idx ON graphql_audit_logs(request_id);
CREATE INDEX graphql_audit_logs__response_status_idx ON graphql_audit_logs(response_status);
CREATE INDEX graphql_audit_logs__client_ip_idx ON graphql_audit_logs(client_ip);

-- Index for analytics queries on user activity over time
CREATE INDEX graphql_audit_logs__user_time_idx ON graphql_audit_logs(user_id, executed_at);

-- Index for security audits of failed operations
CREATE INDEX graphql_audit_logs__errors_idx ON graphql_audit_logs(response_status, executed_at)
    WHERE response_status = 'error';
