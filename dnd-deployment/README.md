# DND Private GKE Deployment Guide

## 🔒 Security Converter - NATO Classification System

**DND-Compliant Private GKE Deployment (No External IPs)**

This guide provides step-by-step instructions for deploying the Security Converter API to Google Kubernetes Engine (GKE) in a **completely private environment** that meets DND security requirements.

## 🎯 Key Features

- ✅ **No External IPs** on cluster nodes (DND compliant)
- ✅ **Internal Load Balancer** only (no external access)
- ✅ **Private Container Registry** (Artifact Registry)
- ✅ **VPC-Native Networking** with private subnets
- ✅ **Cloud NAT** for outbound connectivity only
- ✅ **NATO Security Classification** conversion capabilities

## 📋 Prerequisites

```bash
# Install required tools
gcloud components install kubectl
gcloud components install gke-gcloud-auth-plugin

# Authenticate
gcloud auth login
gcloud config set project admds-edip-datasandbox
```

## 🚀 Deployment Steps

### Step 1: Create Private GKE Cluster

```bash
cd dnd-deployment
chmod +x *.sh

# Creates VPC, private subnets, Cloud NAT, and private GKE cluster
./01-setup-private-gke.sh
```

**What this does:**
- Creates custom VPC with private subnets
- Sets up Cloud NAT for outbound internet (image pulls, etc.)
- Creates private GKE cluster with **NO external IPs**
- Configures firewall rules for internal communication only

### Step 2: Build and Push Container Image

```bash
# Builds your Rust application and pushes to private Artifact Registry
./02-build-and-push.sh
```

**What this does:**
- Creates private Artifact Registry repository
- Builds optimized Docker image using `Dockerfile.slim`
- Pushes to your private registry (no external dependencies)

### Step 3: Deploy Application

```bash
# Deploys PostgreSQL, API, and internal load balancer
./03-deploy-to-gke.sh
```

**What this does:**
- Deploys PostgreSQL with persistent storage
- Deploys Security Converter API using private image
- Creates **internal load balancer** (no external IP)
- Waits for all components to be ready

## 🔍 Verification

### Check Deployment Status
```bash
kubectl get pods
kubectl get services
kubectl get ingress
```

### Check Application Logs
```bash
kubectl logs -l app=api
kubectl logs -l app=postgres
```

### Get Internal IP Address
```bash
kubectl get ingress api-ingress-internal -o jsonpath='{.status.loadBalancer.ingress[0].ip}'
```

### Test API (from within VPC)
```bash
# Get internal IP
INTERNAL_IP=$(kubectl get ingress api-ingress-internal -o jsonpath='{.status.loadBalancer.ingress[0].ip}')

# Test endpoints
curl http://$INTERNAL_IP/
curl http://$INTERNAL_IP/graphql
```

## 🏗️ Architecture

```
DND Environment (No External IPs)
├── Private VPC (10.0.0.0/20)
├── Private GKE Cluster
│   ├── PostgreSQL (persistent storage)
│   ├── Security Converter API
│   └── Internal Load Balancer
├── Cloud NAT (outbound only)
└── Private Artifact Registry
```

## 🔧 Configuration Files

### Environment Variables (`.env`)
Contains all configuration for the deployment:
- Database connection strings
- Security keys  
- Admin credentials
- GCP project settings

### Kubernetes Manifests (`k8s-manifests/`)
- `postgres.yaml` - PostgreSQL database with persistence
- `api.yaml` - Security Converter API with private image
- `ingress-internal.yaml` - Internal load balancer (DND compliant)

## 🛡️ DND Compliance Checklist

- ✅ **No external IPs** on cluster nodes
- ✅ **Internal load balancer** only (no external access)
- ✅ **Private container registry** (no external dependencies)
- ✅ **VPC-native networking** (isolated network)
- ✅ **Encrypted persistent storage** for database
- ✅ **Security keys** managed via ConfigMap/Secrets
- ✅ **Resource limits** and health checks configured

## 🔧 Troubleshooting

### Cluster Creation Fails
```bash
# Check constraints
gcloud resource-manager org-policies list --project=admds-edip-datasandbox

# Verify no external IP constraint
gcloud compute instances list
```

### Image Pull Errors
```bash
# Check Artifact Registry permissions
gcloud artifacts repositories list
gcloud artifacts repositories get-iam-policy security-converter-repo --location=northamerica-northeast1
```

### Ingress Not Getting IP
```bash
# Check ingress controller
kubectl get events --sort-by=.lastTimestamp
kubectl describe ingress api-ingress-internal
```

### Application Won't Start
```bash
# Check logs and configuration
kubectl logs -l app=api --follow
kubectl describe pod -l app=api
```

## 🗑️ Cleanup

```bash
# Delete cluster and all resources
kubectl delete -f k8s-manifests/
gcloud container clusters delete security-converter-private --zone=northamerica-northeast1-a
gcloud compute networks delete security-converter-vpc
gcloud artifacts repositories delete security-converter-repo --location=northamerica-northeast1
```

## 📞 Support

- **GraphQL Playground**: `http://[INTERNAL-IP]/graphql`
- **Health Check**: `http://[INTERNAL-IP]/`
- **Application Logs**: `kubectl logs -l app=api`

---

**🔒 This deployment is fully DND-compliant with no external IP addresses.**