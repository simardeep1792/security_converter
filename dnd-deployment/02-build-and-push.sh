#!/bin/bash

# Build and push container image to Artifact Registry
# Enhanced with better tagging, caching, and error handling

set -euo pipefail
source config.env

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if repository exists
repository_exists() {
    gcloud artifacts repositories describe "$REGISTRY_NAME" --location="$REGION" --quiet >/dev/null 2>&1
}

# Check if image exists in registry
image_exists() {
    local image_uri="$1"
    gcloud artifacts docker images describe "$image_uri" --quiet >/dev/null 2>&1
}

log_info "BUILDING AND PUSHING SECURITY CONVERTER IMAGE"
log_info "Registry: $REGISTRY_NAME"
log_info "Image: $IMAGE_NAME"
log_info "Tag: $IMAGE_TAG"

# Generate additional tags
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
GIT_COMMIT=""
if git rev-parse --git-dir > /dev/null 2>&1; then
    GIT_COMMIT=$(git rev-parse --short HEAD)
    log_info "Git commit: $GIT_COMMIT"
fi

# Set project
gcloud config set project "$PROJECT_ID"

# Create Artifact Registry repository (idempotent)
if repository_exists; then
    log_warn "Artifact Registry repository $REGISTRY_NAME already exists, skipping creation"
else
    log_info "Creating Artifact Registry repository: $REGISTRY_NAME"
    gcloud artifacts repositories create "$REGISTRY_NAME" \
        --repository-format=docker \
        --location="$REGION" \
        --description="Security Converter Container Repository" \
        --quiet
fi

# Configure Docker authentication
log_info "Configuring Docker authentication for Artifact Registry..."
gcloud auth configure-docker "$REGION-docker.pkg.dev" --quiet

# Build image URIs
BASE_URI="$REGION-docker.pkg.dev/$PROJECT_ID/$REGISTRY_NAME/$IMAGE_NAME"
IMAGE_URI_LATEST="$BASE_URI:latest"
IMAGE_URI_TIMESTAMP="$BASE_URI:$TIMESTAMP"

# Add git commit tag if available
if [ -n "$GIT_COMMIT" ]; then
    IMAGE_URI_COMMIT="$BASE_URI:$GIT_COMMIT"
fi

# Check if we need to build (skip if image already exists)
if [ "$IMAGE_TAG" != "latest" ] && image_exists "$BASE_URI:$IMAGE_TAG"; then
    log_warn "Image $BASE_URI:$IMAGE_TAG already exists, skipping build"
    log_info "Existing image URI: $BASE_URI:$IMAGE_TAG"
    echo ""
    log_info "Next: Run 03-deploy-to-gke.sh"
    exit 0
fi

# Change to project root directory
log_info "Changing to project root directory..."
cd "$(dirname "$0")/.."

# Verify Dockerfile exists
if [ ! -f "Dockerfile.slim" ]; then
    log_error "Dockerfile.slim not found in project root"
    exit 1
fi

# Build the image using Dockerfile.slim (optimized version)
log_info "Building Docker image with Dockerfile.slim..."
log_info "Using Docker BuildKit for improved caching..."

# Enable BuildKit for better caching and performance
export DOCKER_BUILDKIT=1

# Build with multiple tags in one command for efficiency
BUILD_ARGS=""
if [ -n "$GIT_COMMIT" ]; then
    BUILD_ARGS="--build-arg GIT_COMMIT=$GIT_COMMIT"
fi

BUILD_TAGS="-t $IMAGE_NAME:latest -t $IMAGE_URI_LATEST -t $IMAGE_URI_TIMESTAMP"
if [ -n "$GIT_COMMIT" ]; then
    BUILD_TAGS="$BUILD_TAGS -t $IMAGE_URI_COMMIT"
fi

log_info "Building with tags: latest, $TIMESTAMP$([ -n "$GIT_COMMIT" ] && echo ", $GIT_COMMIT")"

docker build -f Dockerfile.slim $BUILD_ARGS $BUILD_TAGS .

# Return to deployment directory
cd dnd-deployment

# Push images to Artifact Registry
log_info "Pushing images to Artifact Registry..."

# Push latest tag
log_info "Pushing latest tag..."
docker push "$IMAGE_URI_LATEST"

# Push timestamp tag
log_info "Pushing timestamp tag..."
docker push "$IMAGE_URI_TIMESTAMP"

# Push commit tag if available
if [ -n "$GIT_COMMIT" ]; then
    log_info "Pushing commit tag..."
    docker push "$IMAGE_URI_COMMIT"
fi

# Verify images in registry
log_info "Verifying images in Artifact Registry..."
gcloud artifacts docker images list "$REGION-docker.pkg.dev/$PROJECT_ID/$REGISTRY_NAME" \
    --include-tags \
    --sort-by="CREATE_TIME" \
    --limit=10

# Create image URI for deployment (use commit tag if available, otherwise timestamp)
DEPLOYMENT_IMAGE_URI="$IMAGE_URI_LATEST"
if [ -n "$GIT_COMMIT" ]; then
    DEPLOYMENT_IMAGE_URI="$IMAGE_URI_COMMIT"
fi

log_info "✓ IMAGE BUILD AND PUSH COMPLETE!"
echo ""
log_info "Available image URIs:"
log_info "  Latest: $IMAGE_URI_LATEST"
log_info "  Timestamp: $IMAGE_URI_TIMESTAMP"
if [ -n "$GIT_COMMIT" ]; then
    log_info "  Commit: $IMAGE_URI_COMMIT"
fi
echo ""
log_info "Deployment will use: $DEPLOYMENT_IMAGE_URI"
echo ""

# Update IMAGE_TAG in config.env for deployment script
if [ -n "$GIT_COMMIT" ]; then
    DEPLOY_TAG="$GIT_COMMIT"
else
    DEPLOY_TAG="$TIMESTAMP"
fi

# Create a deployment-specific config file
cat > deployment-config.env << EOF
# Auto-generated deployment configuration
# This file is created by 02-build-and-push.sh

DEPLOYMENT_IMAGE_URI="$DEPLOYMENT_IMAGE_URI"
DEPLOYMENT_TAG="$DEPLOY_TAG"
BUILD_TIMESTAMP="$TIMESTAMP"
EOF

log_info "Created deployment-config.env with image details"
echo ""
log_info "Next: Run 03-deploy-to-gke.sh"