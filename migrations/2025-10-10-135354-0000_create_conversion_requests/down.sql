-- Drop conversion_requests table
DROP INDEX IF EXISTS conversion_requests__completed_at_idx;
DROP INDEX IF EXISTS conversion_requests__created_at_idx;
DROP INDEX IF EXISTS conversion_requests__source_nation_code_idx;
DROP INDEX IF EXISTS conversion_requests__metadata_id_idx;
DROP INDEX IF EXISTS conversion_requests__data_object_id_idx;
DROP INDEX IF EXISTS conversion_requests__authority_id_idx;
DROP INDEX IF EXISTS conversion_requests__creator_id_idx;
DROP TABLE IF EXISTS conversion_requests;
