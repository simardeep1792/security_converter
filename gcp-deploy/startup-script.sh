#!/bin/bash
# GCP VM Startup Script for Security Classification Converter
# This script runs on first boot to set up Docker and deploy the application

set -e

# Log everything
exec > >(tee /var/log/startup-script.log) 2>&1
echo "Starting deployment at $(date)"

# Update system
apt-get update
apt-get upgrade -y

# Install Docker
apt-get install -y apt-transport-https ca-certificates curl gnupg lsb-release

# Add Docker's official GPG key
curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg

# Set up Docker repository
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/debian $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null

# Install Docker Engine
apt-get update
apt-get install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin

# Start and enable Docker
systemctl start docker
systemctl enable docker

# Create app directory
mkdir -p /opt/security-converter
cd /opt/security-converter

# Create .env file from metadata (set these in VM metadata)
cat > .env << 'ENVEOF'
DATABASE_URL=postgres://christopherallison:12345@db/security_classification_converter?sslmode=disable
SECRET_KEY=CHANGE_ME_32_CHAR_SECRET_KEY_123
PASSWORD_SECRET_KEY=CHANGE_ME_32_CHAR_PASSWORD_KEY12
JWT_SECRET_KEY=CHANGE_ME_32_CHAR_JWT_SECRET_123
ADMIN_EMAIL=admin@example.com
ADMIN_PASSWORD=CHANGE_ME_SECURE_PASSWORD
ADMIN_NAME="Admin User"
ENVEOF

# Create docker-compose.yml
cat > docker-compose.yml << 'COMPOSEEOF'
services:
  db:
    image: postgres:latest
    restart: always
    environment:
      - POSTGRES_USER=christopherallison
      - POSTGRES_PASSWORD=12345
      - POSTGRES_DB=security_classification_converter
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "127.0.0.1:5432:5432"

  api:
    image: ghcr.io/YOUR_REPO/security-converter:latest
    # Or build locally:
    # build:
    #   dockerfile: Dockerfile.slim
    #   context: .
    restart: always
    environment:
      - HOST=0.0.0.0
      - PORT=8080
      - DATABASE_URL=postgres://christopherallison:12345@db/security_classification_converter?sslmode=disable
    env_file:
      - .env
    depends_on:
      - db
    ports:
      - "127.0.0.1:8080:8080"

volumes:
  postgres_data:
COMPOSEEOF

echo "Startup script completed at $(date)"
echo "To start the application, run: cd /opt/security-converter && docker compose up -d"
