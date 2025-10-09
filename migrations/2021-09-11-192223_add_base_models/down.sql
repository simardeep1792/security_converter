-- This file should undo anything in `up.sql`

DROP TABLE IF EXISTS metadata;
DROP TABLE IF EXISTS data_objects;

DROP TABLE IF EXISTS authorities;
DROP TABLE IF EXISTS nations;

DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS valid_roles;

DROP TYPE IF EXISTS access_level_enum;
DROP TYPE IF EXISTS skill_domain;


