-- Your SQL goes here

CREATE TYPE access_level_enum AS ENUM (
    'adminstrator',
    'analyst',
    'employee',
    'research',
    'open'
);

CREATE TABLE IF NOT EXISTS valid_roles (
   role VARCHAR(64) PRIMARY KEY
);

INSERT INTO valid_roles (role) VALUES
    ('ADMIN'),
    ('USER'),
    ('ANALYST'),
    ('OPERATOR');

CREATE TABLE IF NOT EXISTS users (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    hash VARCHAR(255) NOT NULL,
    email VARCHAR(128) UNIQUE NOT NULL UNIQUE,
    role VARCHAR(64) REFERENCES valid_roles (role) ON UPDATE CASCADE DEFAULT 'USER' NOT NULL,
    name VARCHAR(256) NOT NULL,
    access_level VARCHAR(64) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    access_key VARCHAR(256) NOT NULL,
    approved_by_user_uid UUID
);

CREATE UNIQUE INDEX users__email_idx ON users(email);

-- Your SQL goes here

CREATE TABLE IF NOT EXISTS nations (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    creator_id UUID NOT NULL,
        FOREIGN KEY(creator_id)
        REFERENCES users(id) ON DELETE RESTRICT,
    nation_code VARCHAR(3) NOT NULL,
    nation_name VARCHAR(128) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS authorities (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    creator_id UUID NOT NULL,
        FOREIGN KEY(creator_id)
        REFERENCES users(id) ON DELETE RESTRICT,
    nation_id UUID NOT NULL,
        FOREIGN KEY(nation_id)
        REFERENCES nations(id) ON DELETE RESTRICT,
    name VARCHAR(256) NOT NULL,
    email VARCHAR(128) NOT NULL,
    phone VARCHAR(32) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP DEFAULT NULL
);

CREATE INDEX authorities__creator_id_idx ON authorities(creator_id);
CREATE INDEX authorities__nation_id_idx ON authorities(nation_id);

CREATE TABLE IF NOT EXISTS data_objects (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    creator_id UUID NOT NULL,
        FOREIGN KEY(creator_id)
        REFERENCES users(id) ON DELETE RESTRICT,
    title VARCHAR(512) NOT NULL,
    description TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX data_objects__creator_id_idx ON data_objects(creator_id);
CREATE INDEX data_objects__title_idx ON data_objects(title);

CREATE TABLE IF NOT EXISTS metadata (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    data_object_id UUID NOT NULL,
        FOREIGN KEY (data_object_id)
        REFERENCES data_objects(id) ON DELETE CASCADE,
    domain VARCHAR(256) NOT NULL,
    tags TEXT[] NOT NULL DEFAULT '{"joint_forces"}',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX metadata__domain_idx ON metadata(domain);
CREATE INDEX metadata__id_idx ON metadata(data_object_id)