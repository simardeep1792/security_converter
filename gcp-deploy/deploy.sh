#!/bin/bash
# Quick deployment script for Security Classification Converter to GCP
# Run this from your local machine with gcloud configured

set -e

# Configuration - CHANGE THESE
export PROJECT_ID="sandbox-caf-compute-hub"
export ZONE="northamerica-northeast1-a"
export VM_NAME="security-converter-vm"
export NETWORK="default"  # Change if using custom VPC

echo "=== Security Converter GCP Deployment ==="
echo "Project: $PROJECT_ID"
echo "Zone: $ZONE"
echo "VM: $VM_NAME"
echo ""

# Set project
gcloud config set project $PROJECT_ID

# Step 1: Enable APIs
echo ">>> Enabling required APIs..."
gcloud services enable compute.googleapis.com --quiet
gcloud services enable iap.googleapis.com --quiet

# Step 2: Create firewall rule (ignore if exists)
echo ">>> Creating firewall rule for IAP..."
gcloud compute firewall-rules create allow-iap-tunnel \
    --direction=INGRESS \
    --priority=1000 \
    --network=$NETWORK \
    --action=ALLOW \
    --rules=tcp:22,tcp:8080 \
    --source-ranges=35.235.240.0/20 \
    --target-tags=iap-enabled \
    2>/dev/null || echo "Firewall rule already exists, skipping..."

# Step 3: Create VM
echo ">>> Creating VM instance..."
gcloud compute instances create $VM_NAME \
    --project=$PROJECT_ID \
    --zone=$ZONE \
    --machine-type=e2-medium \
    --image-family=debian-12 \
    --image-project=debian-cloud \
    --boot-disk-size=20GB \
    --boot-disk-type=pd-balanced \
    --no-address \
    --tags=iap-enabled \
    --metadata-from-file=startup-script=startup-script.sh \
    --scopes=cloud-platform

echo ">>> Waiting for VM to initialize (60 seconds)..."
sleep 60

# Step 4: Package and copy source files
echo ">>> Packaging source files..."
cd "$(dirname "$0")/.."

tar -czvf /tmp/security-converter.tar.gz \
    Cargo.toml Cargo.lock \
    graphql_api errors migrations templates \
    Dockerfile.slim .env

echo ">>> Copying files to VM via IAP tunnel..."
gcloud compute scp /tmp/security-converter.tar.gz $VM_NAME:/tmp/ \
    --zone=$ZONE \
    --tunnel-through-iap

# Step 5: Extract and start on VM
echo ">>> Setting up application on VM..."
gcloud compute ssh $VM_NAME --zone=$ZONE --tunnel-through-iap --command="
    # Wait for startup script to complete Docker installation
    echo 'Waiting for Docker installation...'
    while ! command -v docker &> /dev/null; do
        sleep 10
        echo 'Still waiting for Docker...'
    done

    # Extract files
    cd /opt/security-converter
    sudo tar -xzvf /tmp/security-converter.tar.gz

    # Build and start
    echo 'Building and starting containers...'
    sudo docker compose up -d --build

    echo 'Waiting for services to start...'
    sleep 30

    # Show status
    sudo docker compose ps
"

echo ""
echo "=== Deployment Complete ==="
echo ""
echo "To access the application, run:"
echo "  gcloud compute start-iap-tunnel $VM_NAME 8080 --local-host-port=localhost:8080 --zone=$ZONE"
echo ""
echo "Then open: http://localhost:8080/playground"
echo ""
echo "To SSH into the VM:"
echo "  gcloud compute ssh $VM_NAME --zone=$ZONE --tunnel-through-iap"
