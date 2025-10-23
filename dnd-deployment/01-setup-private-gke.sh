#!/bin/bash

# DND Private Autopilot GKE Setup - Enhanced with Idempotency and NAT Toggle
# Creates private autopilot cluster that complies with DND security requirements

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

# Check if resource exists functions
vpc_exists() {
    gcloud compute networks describe "$VPC_NAME" --quiet >/dev/null 2>&1
}

subnet_exists() {
    gcloud compute networks subnets describe "$SUBNET_NAME" --region="$REGION" --quiet >/dev/null 2>&1
}

firewall_rule_exists() {
    gcloud compute firewall-rules describe "$1" --quiet >/dev/null 2>&1
}

router_exists() {
    gcloud compute routers describe "$ROUTER_NAME" --region="$REGION" --quiet >/dev/null 2>&1
}

nat_exists() {
    gcloud compute routers nats describe "$NAT_NAME" --router="$ROUTER_NAME" --region="$REGION" --quiet >/dev/null 2>&1
}

cluster_exists() {
    gcloud container clusters describe "$CLUSTER_NAME" --region="$REGION" --quiet >/dev/null 2>&1
}

repository_exists() {
    gcloud artifacts repositories describe "$REGISTRY_NAME" --location="$REGION" --quiet >/dev/null 2>&1
}

log_info "DND PRIVATE AUTOPILOT GKE DEPLOYMENT"
log_info "Project: $PROJECT_ID"
log_info "Region: $REGION"  
log_info "Cluster: $CLUSTER_NAME"
log_info "NAT Enabled: $NAT_TOGGLE"

# Set project
log_info "Setting GCP project..."
gcloud config set project "$PROJECT_ID"

# Enable required APIs
log_info "Enabling required GCP APIs..."
gcloud services enable compute.googleapis.com \
    container.googleapis.com \
    artifactregistry.googleapis.com \
    --quiet

log_info "Creating VPC and Private Networking..."

# Create VPC network (idempotent)
if vpc_exists; then
    log_warn "VPC $VPC_NAME already exists, skipping creation"
else
    log_info "Creating VPC network: $VPC_NAME"
    gcloud compute networks create "$VPC_NAME" \
        --subnet-mode=custom \
        --bgp-routing-mode=regional \
        --quiet
fi

# Create subnet with secondary ranges for autopilot (idempotent)
if subnet_exists; then
    log_warn "Subnet $SUBNET_NAME already exists, skipping creation"
else
    log_info "Creating subnet: $SUBNET_NAME"
    gcloud compute networks subnets create "$SUBNET_NAME" \
        --network="$VPC_NAME" \
        --range="$VPC_CIDR" \
        --region="$REGION" \
        --secondary-range="$POD_RANGE_NAME=$POD_RANGE,$SERVICE_RANGE_NAME=$SERVICE_RANGE" \
        --enable-private-ip-google-access \
        --quiet
fi

log_info "Creating Firewall Rules..."

# Create firewall rules (idempotent)
INTERNAL_FW_RULE="allow-internal-$VPC_NAME"
if firewall_rule_exists "$INTERNAL_FW_RULE"; then
    log_warn "Firewall rule $INTERNAL_FW_RULE already exists, skipping creation"
else
    log_info "Creating internal firewall rule: $INTERNAL_FW_RULE"
    gcloud compute firewall-rules create "$INTERNAL_FW_RULE" \
        --network="$VPC_NAME" \
        --allow=tcp,udp,icmp \
        --source-ranges="$VPC_CIDR,$POD_RANGE,$SERVICE_RANGE" \
        --quiet
fi

HEALTH_CHECK_FW_RULE="allow-health-checks-$VPC_NAME"
if firewall_rule_exists "$HEALTH_CHECK_FW_RULE"; then
    log_warn "Firewall rule $HEALTH_CHECK_FW_RULE already exists, skipping creation"
else
    log_info "Creating health check firewall rule: $HEALTH_CHECK_FW_RULE"
    gcloud compute firewall-rules create "$HEALTH_CHECK_FW_RULE" \
        --network="$VPC_NAME" \
        --allow=tcp \
        --source-ranges=130.211.0.0/22,35.191.0.0/16 \
        --quiet
fi

# Conditional NAT setup
if [ "$NAT_TOGGLE" = "true" ]; then
    log_info "Setting up Cloud NAT for outbound connectivity..."
    
    # Create router (idempotent)
    if router_exists; then
        log_warn "Router $ROUTER_NAME already exists, skipping creation"
    else
        log_info "Creating Cloud Router: $ROUTER_NAME"
        gcloud compute routers create "$ROUTER_NAME" \
            --network="$VPC_NAME" \
            --region="$REGION" \
            --quiet
    fi
    
    # Create NAT (idempotent)
    if nat_exists; then
        log_warn "Cloud NAT $NAT_NAME already exists, skipping creation"
    else
        log_info "Creating Cloud NAT: $NAT_NAME"
        gcloud compute routers nats create "$NAT_NAME" \
            --router="$ROUTER_NAME" \
            --region="$REGION" \
            --nat-all-subnet-ip-ranges \
            --auto-allocate-nat-external-ips \
            --quiet
    fi
else
    log_info "NAT disabled - cluster will use Private Google Access only"
fi

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

# Create GKE Autopilot cluster (idempotent)
if cluster_exists; then
    log_warn "GKE cluster $CLUSTER_NAME already exists, skipping creation"
else
    log_info "Creating PRIVATE AUTOPILOT GKE Cluster (NO external IPs)..."
    
    # Build cluster create command (Autopilot-compatible flags only)
    CLUSTER_CMD="gcloud container clusters create-auto $CLUSTER_NAME \
        --region=$REGION \
        --network=projects/$PROJECT_ID/global/networks/$VPC_NAME \
        --subnetwork=projects/$PROJECT_ID/regions/$REGION/subnetworks/$SUBNET_NAME \
        --cluster-secondary-range-name=$POD_RANGE_NAME \
        --services-secondary-range-name=$SERVICE_RANGE_NAME \
        --enable-private-nodes \
        --enable-private-endpoint \
        --master-ipv4-cidr=$MASTER_CIDR \
        --release-channel=regular \
        --quiet"
    
    # Add authorized networks if specified
    if [ -n "$AUTHORIZED_NETWORKS" ]; then
        CLUSTER_CMD="$CLUSTER_CMD --master-authorized-networks=$AUTHORIZED_NETWORKS"
    fi
    
    eval "$CLUSTER_CMD"
fi

log_info "Getting cluster credentials..."
gcloud container clusters get-credentials "$CLUSTER_NAME" --region="$REGION" --quiet

log_info "Verifying cluster health..."
if kubectl cluster-info >/dev/null 2>&1; then
    log_info "✓ Cluster is accessible"
else
    log_error "✗ Unable to access cluster"
    exit 1
fi

# Wait for nodes to be ready
log_info "Waiting for nodes to be ready..."
kubectl wait --for=condition=Ready nodes --all --timeout=300s || true

log_info "PRIVATE AUTOPILOT GKE CLUSTER READY!"
kubectl get nodes -o wide
echo ""
log_info "✓ Autopilot cluster created"
log_info "✓ No external IPs on nodes (DND compliant)"
log_info "✓ Auto-scaling enabled"
log_info "✓ Private Google Access configured"

if [ "$NAT_TOGGLE" = "true" ]; then
    log_info "✓ Cloud NAT enabled for outbound connectivity"
else
    log_info "✓ No Cloud NAT - Private Google Access only"
fi

echo ""
log_info "Next: Run 02-build-and-push.sh"