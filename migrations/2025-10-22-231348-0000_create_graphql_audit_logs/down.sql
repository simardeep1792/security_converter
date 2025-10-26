-- Rollback GraphQL Audit Logs table

DROP TABLE IF EXISTS graphql_audit_logs;
DROP TYPE IF EXISTS graphql_operation_type;
