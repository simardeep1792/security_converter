#!/bin/bash

# DND Private Autopilot GKE Setup - No External IPs
# Creates private autopilot cluster that complies with DND security requirements

set -e
source config.env

echo "🔒 DND PRIVATE AUTOPILOT GKE DEPLOYMENT"
echo "Project: $PROJECT_ID"
echo "Region: $REGION"  
echo "Cluster: $CLUSTER_NAME"

# Set project
gcloud config set project $PROJECT_ID

echo "📡 Creating VPC and Private Networking..."

# Create VPC network
gcloud compute networks create $VPC_NAME \
    --subnet-mode=custom \
    --bgp-routing-mode=regional

# Create subnet with secondary ranges for autopilot
gcloud compute networks subnets create $SUBNET_NAME \
    --network=$VPC_NAME \
    --range=10.0.0.0/20 \
    --region=$REGION \
    --secondary-range=pods=10.4.0.0/14,services=10.8.0.0/20 \
    --enable-private-ip-google-access

echo "🛡️ Creating Firewall Rules..."
gcloud compute firewall-rules create allow-internal-$VPC_NAME \
    --network=$VPC_NAME \
    --allow=tcp,udp,icmp \
    --source-ranges=10.0.0.0/8

gcloud compute firewall-rules create allow-health-checks-$VPC_NAME \
    --network=$VPC_NAME \
    --allow=tcp \
    --source-ranges=130.211.0.0/22,35.191.0.0/16

echo "🌐 Setting up Cloud NAT for outbound connectivity..."
gcloud compute routers create $ROUTER_NAME \
    --network=$VPC_NAME \
    --region=$REGION

gcloud compute routers nats create $NAT_NAME \
    --router=$ROUTER_NAME \
    --region=$REGION \
    --nat-all-subnet-ip-ranges \
    --auto-allocate-nat-external-ips

echo "🚢 Creating PRIVATE AUTOPILOT GKE Cluster (NO external IPs)..."
gcloud container clusters create-auto $CLUSTER_NAME \
    --region=$REGION \
    --network="projects/$PROJECT_ID/global/networks/$VPC_NAME" \
    --subnetwork="projects/$PROJECT_ID/regions/$REGION/subnetworks/$SUBNET_NAME" \
    --cluster-secondary-range-name=pods \
    --services-secondary-range-name=services \
    --enable-private-nodes \
    --enable-private-endpoint \
    --master-ipv4-cidr=172.16.0.0/28 \
    --enable-ip-alias \
    --release-channel="regular"

echo "🔑 Getting cluster credentials..."
gcloud container clusters get-credentials $CLUSTER_NAME --region=$REGION

echo "✅ PRIVATE AUTOPILOT GKE CLUSTER READY!"
kubectl get nodes
echo "✅ Autopilot cluster created"
echo "✅ No external IPs on nodes (DND compliant)"
echo "✅ Auto-scaling enabled"
echo ""
echo "Next: Run 02-build-and-push.sh"