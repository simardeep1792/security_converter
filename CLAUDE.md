# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based GraphQL API middleware designed for NATO security classification conversions. It provides an auditable trail for converting security classifications between NATO nations using the NATO standard as a "Rosetta Stone".

## Architecture

- **Rust Workspace**: Multi-crate workspace with `graphql_api` as the main service and `errors` as a shared error handling crate
- **GraphQL API**: Built with `async-graphql` and `actix-web` for the web server
- **Database**: PostgreSQL with Diesel ORM for migrations and queries
- **Authentication**: JWT-based with argon2 password hashing
- **Containerization**: Multi-stage Docker builds with different optimization levels

### Key Components

- `graphql_api/src/models/`: Core data models including User, Nation, Classification, Authority
- `graphql_api/src/graphql/`: GraphQL schema with queries and mutations
- `graphql_api/src/handlers/`: HTTP request handlers and routing
- `graphql_api/src/database.rs`: Database connection pooling and initialization
- `migrations/`: Diesel database migrations

## Development Commands

### Database Setup
```bash
# Install diesel CLI (if not already installed)
cargo install diesel_cli --no-default-features --features postgres

# Run database migrations
diesel migration run
```

### Development Server
```bash
# Run the main API server
cargo run

# Run from the graphql_api directory
cd graphql_api && cargo run
```

### Docker Development
```bash
# Start database and API with Docker Compose
docker compose up

# Build just the API container
docker compose build security_converter

# Run database only for local development
docker compose up -d db
```

### Environment Setup
Create a `.env` file with:
- `DATABASE_URL=postgres://christopherallison:12345@localhost/classification_transformer?sslmode=disable`
- `SECRET_KEY=32CHARSECRETKEY`
- `PASSWORD_SECRET_KEY=32CHARSECRETKEY`
- `JWT_SECRET_KEY=32CHARSECRETKEY`
- `ADMIN_EMAIL=some_admin@email.com`
- `ADMIN_PASSWORD=ADMINPASSWORD`
- `ADMIN_NAME="Admin Name"`

## Key Configuration Files

- `diesel.toml`: Diesel ORM configuration, schema output to `graphql_api/src/schema.rs`
- `docker-compose.yml`: PostgreSQL database on port 5434, API on port 8080
- `Cargo.toml`: Workspace configuration with graphql_api and errors crates

## Testing and Build

The project uses standard Rust tooling. Check for test scripts in the main `Cargo.toml` or individual crate manifests.

## Database Schema

The application uses Diesel migrations located in `migrations/`. The schema is auto-generated to `graphql_api/src/schema.rs`. Models are organized by domain (user management, classification system, etc.).

## GraphQL Endpoint

When running, the GraphQL playground is available at the `/graphql` endpoint with GraphiQL interface enabled for development.

## Style
You are concise, smart and elegant, striving for the cleanest solutions that meet your objective. Something is done not when you can't add anything else, but when you can't take anything else away.