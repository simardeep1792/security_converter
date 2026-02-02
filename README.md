# NATO Security Classification Converter API

A Rust-based GraphQL API middleware designed to provide an auditable trail for security classification conversions between NATO nations, using the NATO standard as a "Rosetta Stone".

## Features

### Implemented
- [x] User models with role-based access control
- [x] Automated Admin Generation on first startup
- [x] JWT-based Authentication and sign-in
- [x] Nation and Authority management
- [x] Classification Schema definitions per nation
- [x] Data Object tagging with unique UUIDs
- [x] Conversion Request tracking with audit trail
- [x] GraphQL API with GraphiQL playground
- [x] Docker containerization with multi-stage builds
- [x] GCP deployment support (with IAP tunneling)

### Planned
- [ ] Blockchain implementation for document tagging
- [ ] Community of Interest management tools
- [ ] Enhanced audit logging

## Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Frontend      │────▶│  GraphQL API    │────▶│   PostgreSQL    │
│   (Optional)    │     │  (Rust/Actix)   │     │   Database      │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                              │
                              ▼
                        ┌─────────────────┐
                        │  Classification │
                        │  Schema Engine  │
                        └─────────────────┘
```

### Tech Stack
- **Language**: Rust (2024 Edition)
- **Web Framework**: Actix-web 4.x
- **GraphQL**: async-graphql 7.x
- **Database**: PostgreSQL 16 with Diesel ORM
- **Authentication**: JWT + Argon2 password hashing
- **Containerization**: Docker with multi-stage builds

## Quick Start

### Prerequisites
- Docker and Docker Compose
- (Optional) Rust toolchain for local development
- (Optional) diesel-cli for migrations

### Using Docker (Recommended)

1. Clone the repository:
```bash
git clone <repository-url>
cd security_converter
```

2. Create `.env` file:
```bash
DATABASE_URL=postgres://christopherallison:12345@localhost:5434/security_classification_converter?sslmode=disable
SECRET_KEY=your_32_character_secret_key_here
PASSWORD_SECRET_KEY=your_32_char_password_secret_key
JWT_SECRET_KEY=your_32_character_jwt_secret_key
ADMIN_EMAIL=admin@example.com
ADMIN_PASSWORD=your_secure_admin_password
ADMIN_NAME="Admin User"
```

3. Start the services:
```bash
docker compose up -d
```

4. Access the API:
- **Home**: http://localhost:8080
- **GraphQL Playground**: http://localhost:8080/playground
- **GraphQL API**: http://localhost:8080/graphql (POST)

### Local Development

1. Install dependencies:
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install diesel-cli
cargo install diesel_cli --no-default-features --features postgres
```

2. Start PostgreSQL:
```bash
docker compose up -d db
```

3. Run migrations:
```bash
diesel migration run
```

4. Start the server:
```bash
cargo run
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | API documentation home page |
| `/playground` | GET | GraphiQL interactive playground |
| `/graphql` | POST | GraphQL API endpoint |
| `/graphql` | GET (WebSocket) | GraphQL subscriptions |

## GraphQL Schema

### Key Types
- **Nation**: NATO member nations with their classification systems
- **Authority**: Organizations authorized to perform classifications
- **ClassificationSchema**: Classification levels (UNCLASSIFIED → TOP SECRET)
- **DataObject**: Documents/data packages with classification metadata
- **ConversionRequest**: Audit trail for classification conversions
- **User**: System users with role-based access

### Example Queries

```graphql
# Get all nations
query {
  allNations {
    id
    name
    code
  }
}

# Get classification schemas for a nation
query {
  classificationSchemasByNation(nationId: "uuid-here") {
    level
    description
  }
}

# Create a conversion request
mutation {
  createConversionRequest(
    sourceNationId: "uuid-source"
    targetNationId: "uuid-target"
    dataObjectId: "uuid-data"
  ) {
    id
    status
    createdAt
  }
}
```

## Production Deployment (GCP)

The production environment runs on GCP with images stored in Google Artifact Registry for fast deployments.

### Quick Deploy (Recommended)

After making changes, deploy to production with:

```bash
./deploy.sh
```

This script:
1. Builds Docker image locally (uses cache for fast rebuilds)
2. Pushes to Google Artifact Registry
3. Pulls and restarts on the VM

| Step | First Time | Subsequent |
|------|------------|------------|
| Build | ~15-20 min | ~30 sec |
| Push | ~30 sec | ~10 sec |
| VM Deploy | ~15 sec | ~15 sec |

### Manual Deployment

```bash
# 1. Build locally
docker build -t northamerica-northeast1-docker.pkg.dev/sandbox-caf-compute-hub/security-converter/api:latest -f Dockerfile.slim .

# 2. Push to registry
docker push northamerica-northeast1-docker.pkg.dev/sandbox-caf-compute-hub/security-converter/api:latest

# 3. Deploy on VM
gcloud compute ssh security-converter-vm --zone=northamerica-northeast1-a --tunnel-through-iap --command="
  cd /opt/security-converter && \
  sudo docker compose pull people-data-api && \
  sudo docker compose up -d people-data-api
"
```

### Access Production

```bash
# Start IAP tunnel (keep running in terminal)
gcloud compute start-iap-tunnel security-converter-vm 8080 \
  --local-host-port=localhost:8080 \
  --zone=northamerica-northeast1-a

# Then open http://localhost:8080/playground
```

### SSH Access

```bash
gcloud compute ssh security-converter-vm \
  --zone=northamerica-northeast1-a \
  --tunnel-through-iap
```

### View Logs

```bash
gcloud compute ssh security-converter-vm --zone=northamerica-northeast1-a --tunnel-through-iap --command="sudo docker compose -f /opt/security-converter/docker-compose.yml logs -f people-data-api"
```

### Infrastructure Details

| Resource | Value |
|----------|-------|
| Project | `sandbox-caf-compute-hub` |
| VM | `security-converter-vm` |
| Zone | `northamerica-northeast1-a` |
| Registry | `northamerica-northeast1-docker.pkg.dev/sandbox-caf-compute-hub/security-converter/api` |
| Access | IAP tunneling (no public IP) |

See [gcp-deploy/README.md](gcp-deploy/README.md) for initial setup instructions.

## Docker Configuration

### Container Images

| Dockerfile | Base Image | Size (approx) | Description |
|------------|------------|---------------|-------------|
| Dockerfile.slim | debian:bookworm-slim | ~450MB | Production (recommended) |
| Dockerfile.simple | rust:latest | ~4GB | Development/debugging |

### Building Manually

```bash
# Build the API image
docker compose build people-data-api

# View logs
docker compose logs -f people-data-api
```

## Project Structure

```
security_converter/
├── graphql_api/           # Main API crate
│   ├── src/
│   │   ├── models/        # Database models
│   │   ├── graphql/       # GraphQL schema, queries, mutations
│   │   ├── handlers/      # HTTP route handlers
│   │   └── database.rs    # Database connection pool
│   └── static/            # Static files (JS, CSS)
├── errors/                # Shared error handling crate
├── migrations/            # Diesel database migrations
├── templates/             # HTML templates (Tera)
├── gcp-deploy/            # GCP deployment scripts
├── kubernetes/            # Kubernetes manifests
└── docker-compose.yml     # Local development setup
```

## Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `DATABASE_URL` | PostgreSQL connection string | Yes |
| `SECRET_KEY` | Application secret (32 chars) | Yes |
| `PASSWORD_SECRET_KEY` | Password hashing secret (32 chars) | Yes |
| `JWT_SECRET_KEY` | JWT signing secret (32 chars) | Yes |
| `ADMIN_EMAIL` | Initial admin email | Yes |
| `ADMIN_PASSWORD` | Initial admin password | Yes |
| `ADMIN_NAME` | Initial admin display name | Yes |
| `HOST` | Server bind address | No (default: 0.0.0.0) |
| `PORT` | Server port | No (default: 8080) |

## Database Schema

The application uses Diesel ORM with PostgreSQL. Migrations are embedded in the binary and run automatically on startup.

Key tables:
- `users` - User accounts and authentication
- `nations` - NATO member nations
- `authorities` - Classification authorities
- `classification_schemas` - Classification levels per nation
- `data_objects` - Tagged documents/data
- `conversion_requests` - Conversion audit trail
- `metadata` - Additional data object metadata

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Submit a pull request

## License

[Add your license here]

## Acknowledgments

- Built with [async-graphql](https://github.com/async-graphql/async-graphql)
- Web framework: [Actix-web](https://actix.rs/)
- ORM: [Diesel](https://diesel.rs/)
