#!/bin/bash

# Build and push container image to Artifact Registry
# Required for private autopilot cluster to access your custom image

set -e
source config.env

echo "BUILDING AND PUSHING SECURITY CONVERTER IMAGE"

# Create Artifact Registry repository
echo "Creating Artifact Registry..."
gcloud artifacts repositories create $REGISTRY_NAME \
   --repository-format=docker \
   --location=$REGION \
   --description="Security Converter Container Repository"

# Configure Docker authentication
gcloud auth configure-docker $REGION-docker.pkg.dev

# Build the image using Dockerfile.slim (optimized version)
echo "Building Docker image..."
cd ..
docker build -f Dockerfile.slim -t $IMAGE_NAME:latest .
cd dnd-deployment

# Tag for Artifact Registry
IMAGE_URI="$REGION-docker.pkg.dev/$PROJECT_ID/$REGISTRY_NAME/$IMAGE_NAME:latest"
docker tag $IMAGE_NAME:latest $IMAGE_URI

# Push to Artifact Registry
echo "Pushing to Artifact Registry..."
docker push $IMAGE_URI

echo "IMAGE READY!"
echo "Image URI: $IMAGE_URI"
echo ""
echo "Next: Run 03-deploy-to-gke.sh"