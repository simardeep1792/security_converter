#!/bin/bash

# Deploy Security Converter to Private Autopilot GKE Cluster
# Uses modified gke-k8s configurations for DND compliance

set -e
source config.env

echo "DEPLOYING TO PRIVATE AUTOPILOT GKE CLUSTER"
echo "Cluster: $CLUSTER_NAME"
echo "Project: $PROJECT_ID"

# Ensure we're connected to the right cluster
gcloud container clusters get-credentials $CLUSTER_NAME --region=$REGION

echo "Verifying cluster status..."
kubectl cluster-info
kubectl get nodes

echo "Step 1: Creating Secrets..."
kubectl apply -f k8s-manifests/secrets.yaml

echo "Step 2: Deploying PostgreSQL Database..."
kubectl apply -f k8s-manifests/postgres.yaml

echo "Waiting for PostgreSQL to be ready..."
kubectl wait --for=condition=ready pod -l app=postgres --timeout=300s

echo "Step 3: Deploying Security Converter API..."
# Update the image URI in api.yaml before applying
IMAGE_URI="$REGION-docker.pkg.dev/$PROJECT_ID/$REGISTRY_NAME/$IMAGE_NAME:latest"
sed "s|northamerica-northeast1-docker.pkg.dev/admds-edip-datasandbox/security-converter-repo/security-converter:latest|$IMAGE_URI|g" k8s-manifests/api.yaml | kubectl apply -f -

echo "Waiting for API to be ready..."
kubectl wait --for=condition=ready pod -l app=api --timeout=600s

echo "Step 4: Setting up Internal Load Balancer..."
kubectl apply -f k8s-manifests/ingress-internal.yaml

echo "Waiting for ingress to get internal IP..."
echo "This may take 5-10 minutes..."

# Wait for ingress to get an IP address
for i in {1..60}; do
    INTERNAL_IP=$(kubectl get ingress api-ingress-internal -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null || echo "")
    if [ ! -z "$INTERNAL_IP" ]; then
        break
    fi
    echo "Waiting for internal IP... ($i/60)"
    sleep 10
done

echo ""
echo "DEPLOYMENT COMPLETE!"
echo "PostgreSQL: Running"
echo "API: Running" 
echo "Internal Load Balancer: Created"
echo "Autopilot Cluster: Auto-scaling enabled"

if [ ! -z "$INTERNAL_IP" ]; then
    echo "Internal IP: $INTERNAL_IP"
    echo "Internal URL: http://$INTERNAL_IP"
    echo "GraphQL Endpoint: http://$INTERNAL_IP/graphql"
else
    echo "Internal IP not ready yet. Check with:"
    echo "   kubectl get ingress api-ingress-internal"
fi

echo ""
echo "Verification Commands:"
echo "kubectl get pods"
echo "kubectl get services"  
echo "kubectl get ingress"
echo "kubectl logs -l app=api"

echo ""
echo "DND COMPLIANCE:"
echo "Autopilot cluster (managed by Google)"
echo "No external IPs on nodes"
echo "Internal load balancer only"
echo "Private container registry"
echo "VPC-native networking"