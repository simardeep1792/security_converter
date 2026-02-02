#!/bin/bash
# Fast deployment script using Google Artifact Registry
# Usage: ./deploy.sh
#
# This builds locally, pushes to registry, and pulls on VM
# First build: ~15-20 minutes (compiling Rust)
# Subsequent builds: ~30 seconds (using Docker cache)
# VM deployment: ~10-20 seconds (just pulling image)

set -e

PROJECT_ID="sandbox-caf-compute-hub"
REGION="northamerica-northeast1"
REPO="security-converter"
IMAGE_NAME="api"
VM_NAME="security-converter-vm"
ZONE="northamerica-northeast1-a"

FULL_IMAGE="${REGION}-docker.pkg.dev/${PROJECT_ID}/${REPO}/${IMAGE_NAME}"

echo "=========================================="
echo "Security Converter Fast Deploy"
echo "=========================================="

# Step 1: Configure Docker for Artifact Registry
echo ""
echo "[1/4] Configuring Docker authentication..."
gcloud auth configure-docker ${REGION}-docker.pkg.dev --quiet

# Step 2: Build the image locally
echo ""
echo "[2/4] Building Docker image locally..."
docker build -t ${FULL_IMAGE}:latest -f Dockerfile.slim .

# Step 3: Push to Artifact Registry
echo ""
echo "[3/4] Pushing image to Artifact Registry..."
docker push ${FULL_IMAGE}:latest

# Step 4: Deploy on VM (pull and restart)
echo ""
echo "[4/4] Deploying on VM..."
gcloud compute ssh ${VM_NAME} --zone=${ZONE} --tunnel-through-iap --command="
cd /opt/security-converter
sudo docker compose pull people-data-api
sudo docker compose up -d people-data-api
sudo docker compose ps
sudo docker compose logs --tail=10 people-data-api
"

echo ""
echo "=========================================="
echo "Deployment complete!"
echo "=========================================="
echo ""
echo "Access via IAP tunnel:"
echo "  gcloud compute start-iap-tunnel ${VM_NAME} 8080 --local-host-port=localhost:8080 --zone=${ZONE}"
echo "  Then open: http://localhost:8080/playground"
