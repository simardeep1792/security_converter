# DND Private GKE Autopilot Deployment Guide

## 🔒 Security Converter - NATO Classification System

**Enhanced DND-Compliant Private GKE Autopilot Deployment**

This guide provides complete instructions for deploying the Security Converter API to Google Kubernetes Engine (GKE) Autopilot in a **fully private, DND-compliant environment** with optional Cloud NAT for controlled egress.

## 🎯 Key Features

- ✅ **GKE Autopilot** - Fully managed, serverless Kubernetes
- ✅ **No External IPs** on cluster nodes (DND compliant)
- ✅ **Internal Load Balancer** only (no external access)
- ✅ **Private Container Registry** (Artifact Registry)
- ✅ **VPC-Native Networking** with defined Pod/Service ranges
- ✅ **Optional Cloud NAT** for controlled outbound connectivity
- ✅ **Idempotent Scripts** safe to re-run
- ✅ **NATO Security Classification** conversion capabilities

## 📋 Prerequisites

```bash
# Install required tools
gcloud components install kubectl
gcloud components install gke-gcloud-auth-plugin

# Authenticate
gcloud auth login
gcloud config set project admds-edip-datasandbox

# Verify Docker is installed and running
docker --version
```

## 🚀 Quick Start

```bash
cd dnd-deployment
chmod +x *.sh

# 1. Create infrastructure and cluster
./01-setup-private-gke.sh

# 2. Build and push application image
./02-build-and-push.sh

# 3. Deploy application
./03-deploy-to-gke.sh
```

## 📖 Detailed Deployment Steps

### Step 1: Infrastructure and Cluster Setup

```bash
./01-setup-private-gke.sh
```

**Creates:**
- Custom VPC with explicit CIDR blocks (10.0.0.0/20)
- Private subnets with secondary ranges for Pods (10.4.0.0/14) and Services (10.8.0.0/20)
- Private Google Access enabled
- Firewall rules for internal communication and health checks
- **Optional Cloud NAT** (controlled by `NAT_TOGGLE` in config.env)
- Private GKE Autopilot cluster with authorized networks
- Artifact Registry repository

**Features:**
- ✅ Idempotent - safe to re-run
- ✅ Error handling and colored output
- ✅ Health checks and verification
- ✅ NAT toggle for air-gapped deployments

### Step 2: Build and Push Container Image

```bash
./02-build-and-push.sh
```

**Features:**
- Multi-tag strategy (latest, timestamp, git commit)
- Docker BuildKit optimization for faster builds
- Image existence checking to avoid unnecessary rebuilds
- Artifact Registry authentication
- Build verification and listing

**Generated Tags:**
- `latest` - For general use
- `YYYYMMDD-HHMMSS` - Timestamp-based
- `git-commit-hash` - For traceability

### Step 3: Application Deployment

```bash
./03-deploy-to-gke.sh
```

**Deploys:**
- Namespace with resource quotas and limits
- PostgreSQL with persistent storage (10Gi)
- Security Converter API with proper health checks
- Internal Load Balancer for private access
- ConfigMaps and Secrets for configuration

**Features:**
- ✅ Comprehensive health checks
- ✅ Proper resource limits for Autopilot
- ✅ Security contexts and non-root containers
- ✅ Automatic image URI substitution
- ✅ Deployment summary generation

## 🔧 Configuration

### Environment Variables (`config.env`)

Key settings you can modify:

```bash
# NAT Configuration (true/false)
NAT_TOGGLE="true"

# Network Configuration
VPC_CIDR="10.0.0.0/20"
POD_RANGE="10.4.0.0/14"
SERVICE_RANGE="10.8.0.0/20"

# Application Configuration
API_REPLICAS="2"
POSTGRES_PVC_SIZE="10Gi"

# Resource Limits (Autopilot optimized)
API_CPU_REQUEST="100m"
API_MEMORY_REQUEST="256Mi"
API_CPU_LIMIT="500m"
API_MEMORY_LIMIT="512Mi"
```

### Switching Between NAT Scenarios

**With NAT (Default):**
```bash
# In config.env
NAT_TOGGLE="true"
```
- Allows outbound internet access for updates/patches
- Minimal egress to Google APIs and container registries

**Without NAT (Air-gapped):**
```bash
# In config.env
NAT_TOGGLE="false"
```
- No outbound internet access
- Uses Private Google Access only
- Requires pre-built images in Artifact Registry

## 🔍 Verification and Monitoring

### Check Deployment Status
```bash
kubectl get pods -n security-converter -o wide
kubectl get services -n security-converter
kubectl get ingress -n security-converter
```

### View Application Logs
```bash
kubectl logs -l app=api -n security-converter --follow
kubectl logs -l app=postgres -n security-converter
```

### Get Internal Access Information
```bash
# Get internal IP
INTERNAL_IP=$(kubectl get ingress api-ingress-internal -n security-converter -o jsonpath='{.status.loadBalancer.ingress[0].ip}')

echo "Health Check: http://$INTERNAL_IP/"
echo "GraphQL Playground: http://$INTERNAL_IP/graphql"
```

### Test API Health (from within VPC)
```bash
# Test basic connectivity
curl -s http://$INTERNAL_IP/ | head

# Test GraphQL endpoint
curl -s -X POST http://$INTERNAL_IP/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "{ __schema { types { name } } }"}' | head
```

## 🏗️ Architecture

```
DND Private Environment
├── VPC (10.0.0.0/20)
│   ├── Private Subnet
│   │   ├── Pod Range (10.4.0.0/14)
│   │   └── Service Range (10.8.0.0/20)
│   └── Optional Cloud NAT
├── GKE Autopilot Cluster
│   ├── Namespace: security-converter
│   ├── PostgreSQL (persistent 10Gi)
│   ├── API (2 replicas, auto-scaling)
│   └── Internal Load Balancer
└── Artifact Registry (private)
```

## 🛡️ DND Compliance Features

| Requirement | Implementation | Status |
|-------------|----------------|--------|
| No External IPs | Private nodes, internal LB only | ✅ |
| Isolated Network | Custom VPC with private subnets | ✅ |
| Controlled Egress | Optional NAT, Private Google Access | ✅ |
| Encrypted Storage | GCP persistent disks (encrypted) | ✅ |
| Resource Limits | Autopilot-optimized constraints | ✅ |
| Security Context | Non-root, dropped capabilities | ✅ |
| Secret Management | Kubernetes secrets (base64) | ✅ |
| Audit Trail | GCP audit logs, deployment summary | ✅ |

## 🔧 Troubleshooting

### Common Issues and Solutions

**1. Cluster Creation Fails**
```bash
# Check organizational policies
gcloud resource-manager org-policies list --project=admds-edip-datasandbox

# Verify API enablement
gcloud services list --enabled
```

**2. Image Pull Errors**
```bash
# Check Artifact Registry access
gcloud artifacts repositories list --location=northamerica-northeast1

# Verify image exists
gcloud artifacts docker images list northamerica-northeast1-docker.pkg.dev/admds-edip-datasandbox/security-converter-repo
```

**3. Pods Not Starting**
```bash
# Check pod status and events
kubectl describe pod -l app=api -n security-converter

# Check resource constraints
kubectl top pods -n security-converter
```

**4. Internal IP Not Assigned**
```bash
# Check ingress status
kubectl describe ingress api-ingress-internal -n security-converter

# Check events
kubectl get events -n security-converter --sort-by=.lastTimestamp
```

**5. Database Connection Issues**
```bash
# Test PostgreSQL directly
kubectl exec -it deployment/postgres-deployment -n security-converter -- psql -U christopherallison -d graphql_api -c '\l'

# Check database logs
kubectl logs deployment/postgres-deployment -n security-converter
```

### Advanced Troubleshooting

**Network Connectivity:**
```bash
# Test from API pod to PostgreSQL
API_POD=$(kubectl get pods -n security-converter -l app=api -o jsonpath='{.items[0].metadata.name}')
kubectl exec -n security-converter $API_POD -- nc -zv postgres-service 5432
```

**Resource Usage:**
```bash
# Check Autopilot resource allocation
kubectl describe nodes
kubectl top nodes
```

## 🗂️ File Structure

```
dnd-deployment/
├── 01-setup-private-gke.sh      # Infrastructure setup
├── 02-build-and-push.sh         # Image build and registry
├── 03-deploy-to-gke.sh          # Application deployment
├── config.env                   # Configuration variables
├── README.md                    # This guide
├── deployment-config.env        # Auto-generated build config
└── deployment-summary.txt       # Auto-generated summary

../gke-k8s/                      # Kubernetes manifests
├── 00-namespace.yaml            # Namespace, quotas, secrets
├── api.yaml                     # API deployment and service
├── postgres.yaml                # PostgreSQL with PVC
└── ingress.yaml                 # Internal load balancer
```

## 🗑️ Cleanup

**Full Cleanup:**
```bash
# Delete applications
kubectl delete namespace security-converter

# Delete cluster
gcloud container clusters delete security-converter-autopilot --region=northamerica-northeast1

# Delete networking
gcloud compute routers nats delete security-converter-nat --router=security-converter-router --region=northamerica-northeast1
gcloud compute routers delete security-converter-router --region=northamerica-northeast1
gcloud compute firewall-rules delete allow-internal-security-converter-vpc
gcloud compute firewall-rules delete allow-health-checks-security-converter-vpc
gcloud compute networks subnets delete security-converter-subnet --region=northamerica-northeast1
gcloud compute networks delete security-converter-vpc

# Delete container registry
gcloud artifacts repositories delete security-converter-repo --location=northamerica-northeast1
```

## 📊 Performance and Scaling

**Autopilot Auto-scaling:**
- Nodes scale automatically based on pod resource requests
- Horizontal Pod Autoscaler (HPA) can be configured for API pods
- Vertical Pod Autoscaler (VPA) recommendations available

**Resource Optimization:**
- API pods: 100m CPU / 256Mi memory (request)
- PostgreSQL: 100m CPU / 256Mi memory (request)
- Persistent storage: 10Gi SSD (expandable)

## 📞 Support and Documentation

**Access URLs (Internal Only):**
- Health Check: `http://[INTERNAL-IP]/`
- GraphQL Playground: `http://[INTERNAL-IP]/graphql`

**Useful Commands:**
```bash
# Quick status check
kubectl get all -n security-converter

# Follow all logs
kubectl logs -f -l app=api -n security-converter

# Port forward for local testing (if authorized networks include your IP)
kubectl port-forward -n security-converter service/api-service 8080:8080
```

**Documentation:**
- [GKE Autopilot Documentation](https://cloud.google.com/kubernetes-engine/docs/concepts/autopilot-overview)
- [Private Clusters](https://cloud.google.com/kubernetes-engine/docs/how-to/private-clusters)
- [Security Best Practices](https://cloud.google.com/kubernetes-engine/docs/how-to/hardening-your-cluster)

---

**🔒 This deployment meets DND security requirements with full private networking and optional controlled egress.**