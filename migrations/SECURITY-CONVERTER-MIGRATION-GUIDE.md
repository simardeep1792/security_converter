# Security Converter - VM to Kubernetes Migration Guide

This guide helps the security-converter team migrate from their current VM deployment to the Aurora Platform GKE cluster.

---

## Current State (VM)

**VM Details:**
- **Name:** security-converter-vm
- **Location:** northamerica-northeast1-a
- **Internal IP:** 10.162.0.14
- **Machine Type:** e2-medium
- **Deployment:** Docker Compose (2 containers: postgres + api)
- **Access:** IAP tunnel on port 8080

**Current Architecture:**
```
User → IAP Tunnel → VM:8080 → Docker (API) → Docker (PostgreSQL)
```

---

## Target State (Kubernetes)

**Cluster Details:**
- **Name:** aurora-platform-cluster
- **Location:** northamerica-northeast1 (3 zones)
- **Deployment:** ArgoCD GitOps
- **Access:** kubectl port-forward or Istio Ingress (optional)

**New Architecture:**
```
User → Ingress/Port-forward → API Pod → PostgreSQL Pod (or Cloud SQL)
```

---

## Migration Steps

### Phase 1: Prepare Your Application (Development Team)

#### 1.1 Create Helm Chart or Kubernetes Manifests

You have two options:

**Option A: Simple Kubernetes Manifests** (Recommended for quick start)

Create a `kubernetes/` directory in your repo with these files:

**`kubernetes/namespace.yaml`**
```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: security-converter
  labels:
    app: security-converter
```

**`kubernetes/secrets.yaml`**
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: security-converter-secrets
  namespace: security-converter
type: Opaque
stringData:
  DATABASE_URL: "postgres://christopherallison:CHANGE_PASSWORD@security-converter-db:5432/security_classification_converter?sslmode=disable"
  SECRET_KEY: "CHANGE_ME_32_CHAR_SECRET_KEY_123"
  PASSWORD_SECRET_KEY: "CHANGE_ME_32_CHAR_PASSWORD_KEY12"
  JWT_SECRET_KEY: "CHANGE_ME_32_CHAR_JWT_SECRET_123"
  ADMIN_EMAIL: "admin@example.com"
  ADMIN_PASSWORD: "CHANGE_ME_SECURE_PASSWORD"
  ADMIN_NAME: "Admin User"
  POSTGRES_USER: "christopherallison"
  POSTGRES_PASSWORD: "CHANGE_PASSWORD"
  POSTGRES_DB: "security_classification_converter"
```

**`kubernetes/database.yaml`**
```yaml
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: postgres-data
  namespace: security-converter
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 20Gi  # Adjust based on your needs
  storageClassName: standard-rwo  # GCP persistent disk
---
apiVersion: v1
kind: Service
metadata:
  name: security-converter-db
  namespace: security-converter
spec:
  selector:
    app: security-converter
    component: database
  ports:
    - port: 5432
      targetPort: 5432
  type: ClusterIP
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: security-converter-db
  namespace: security-converter
spec:
  replicas: 1
  selector:
    matchLabels:
      app: security-converter
      component: database
  template:
    metadata:
      labels:
        app: security-converter
        component: database
    spec:
      containers:
      - name: postgres
        image: postgres:16  # Use specific version
        ports:
        - containerPort: 5432
        env:
        - name: POSTGRES_USER
          valueFrom:
            secretKeyRef:
              name: security-converter-secrets
              key: POSTGRES_USER
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: security-converter-secrets
              key: POSTGRES_PASSWORD
        - name: POSTGRES_DB
          valueFrom:
            secretKeyRef:
              name: security-converter-secrets
              key: POSTGRES_DB
        volumeMounts:
        - name: postgres-storage
          mountPath: /var/lib/postgresql/data
          subPath: postgres  # Avoid mounting to root of volume
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
      volumes:
      - name: postgres-storage
        persistentVolumeClaim:
          claimName: postgres-data
```

**`kubernetes/api.yaml`**
```yaml
---
apiVersion: v1
kind: Service
metadata:
  name: security-converter-api
  namespace: security-converter
spec:
  selector:
    app: security-converter
    component: api
  ports:
    - port: 80
      targetPort: 8080
      name: http
  type: ClusterIP
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: security-converter-api
  namespace: security-converter
spec:
  replicas: 2  # Run 2 replicas for high availability
  selector:
    matchLabels:
      app: security-converter
      component: api
  template:
    metadata:
      labels:
        app: security-converter
        component: api
    spec:
      containers:
      - name: api
        image: ghcr.io/YOUR_REPO/security-converter:latest  # Update this!
        ports:
        - containerPort: 8080
        env:
        - name: HOST
          value: "0.0.0.0"
        - name: PORT
          value: "8080"
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: security-converter-secrets
              key: DATABASE_URL
        - name: SECRET_KEY
          valueFrom:
            secretKeyRef:
              name: security-converter-secrets
              key: SECRET_KEY
        - name: PASSWORD_SECRET_KEY
          valueFrom:
            secretKeyRef:
              name: security-converter-secrets
              key: PASSWORD_SECRET_KEY
        - name: JWT_SECRET_KEY
          valueFrom:
            secretKeyRef:
              name: security-converter-secrets
              key: JWT_SECRET_KEY
        - name: ADMIN_EMAIL
          valueFrom:
            secretKeyRef:
              name: security-converter-secrets
              key: ADMIN_EMAIL
        - name: ADMIN_PASSWORD
          valueFrom:
            secretKeyRef:
              name: security-converter-secrets
              key: ADMIN_PASSWORD
        - name: ADMIN_NAME
          valueFrom:
            secretKeyRef:
              name: security-converter-secrets
              key: ADMIN_NAME
        livenessProbe:
          httpGet:
            path: /health  # Update if your app has different health endpoint
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
```

**Option B: Helm Chart** (Recommended for production)

Create a proper Helm chart structure - contact platform team for template.

#### 1.2 Push Manifests to Git Repository

```bash
# In your security-converter repository
git add kubernetes/
git commit -m "Add Kubernetes manifests for Aurora Platform deployment"
git push origin main
```

---

### Phase 2: Deploy to Aurora Platform (Platform Team Support Needed)

#### 2.1 Request Application Deployment

Contact the Aurora Platform team (you!) with:

1. **Git Repository URL:** https://github.com/YOUR_ORG/security-converter
2. **Path to manifests:** `kubernetes/`
3. **Branch:** `main`
4. **Namespace:** `security-converter`
5. **Secrets:** Provide secure values for secrets.yaml

#### 2.2 Platform Team Creates ArgoCD Application

The platform team will create an ArgoCD Application manifest:

**`security-converter-app.yaml`** (Platform team creates this)
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: security-converter
  namespace: aurora-platform  # Or argocd namespace
spec:
  project: aurora-app  # Or create new project
  source:
    repoURL: https://github.com/YOUR_ORG/security-converter
    targetRevision: main
    path: kubernetes  # Path to manifests in repo
  destination:
    server: https://kubernetes.default.svc
    namespace: security-converter
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
      - CreateNamespace=true
```

Apply it:
```bash
kubectl apply -f security-converter-app.yaml -n aurora-platform
```

---

### Phase 3: Database Migration

#### 3.1 Export Data from VM

On the VM:
```bash
# SSH to VM via IAP
gcloud compute ssh security-converter-vm \
  --project=sandbox-caf-compute-hub \
  --zone=northamerica-northeast1-a \
  --tunnel-through-iap

# Create backup
cd /opt/security-converter
docker compose exec db pg_dump -U christopherallison security_classification_converter > backup.sql

# Exit SSH
exit
```

#### 3.2 Copy Backup to Local Machine

```bash
gcloud compute scp security-converter-vm:/opt/security-converter/backup.sql ./backup.sql \
  --project=sandbox-caf-compute-hub \
  --zone=northamerica-northeast1-a \
  --tunnel-through-iap
```

#### 3.3 Import to Kubernetes PostgreSQL

```bash
# Port forward to database pod
kubectl port-forward -n security-converter svc/security-converter-db 5432:5432

# In another terminal, restore data
psql -h localhost -U christopherallison -d security_classification_converter < backup.sql
```

---

### Phase 4: Access Your Application

#### Option A: Port Forward (Development/Testing)

```bash
kubectl port-forward -n security-converter svc/security-converter-api 8080:80
```

Access at: http://localhost:8080

#### Option B: Ingress (Production - Requires Istio)

Create an Istio VirtualService or Kubernetes Ingress:

**`kubernetes/ingress.yaml`**
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: security-converter
  namespace: security-converter
  annotations:
    kubernetes.io/ingress.class: "nginx"  # Or istio
spec:
  rules:
  - host: security-converter.example.ca  # Update with your domain
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: security-converter-api
            port:
              number: 80
```

---

## Production Considerations

### Use Cloud SQL Instead of In-Cluster PostgreSQL

For production, consider using GCP Cloud SQL:

1. **Create Cloud SQL Instance:**
```bash
gcloud sql instances create security-converter-db \
  --database-version=POSTGRES_16 \
  --tier=db-f1-micro \
  --region=northamerica-northeast1 \
  --project=sandbox-caf-compute-hub
```

2. **Update DATABASE_URL in secrets:**
```
DATABASE_URL: "postgres://user:pass@<CLOUD_SQL_IP>:5432/security_classification_converter?sslmode=require"
```

3. **Use Cloud SQL Proxy sidecar** in your deployment for secure connection.

### Secrets Management

Instead of storing secrets in Git:

1. **Use Google Secret Manager:**
```bash
# Store secret
gcloud secrets create security-converter-db-password \
  --data-file=- \
  --project=sandbox-caf-compute-hub
```

2. **Use External Secrets Operator** (if installed on Aurora Platform) to sync from Secret Manager.

### Monitoring & Logging

Your application will automatically get:
- ✅ **Metrics collected by Prometheus**
- ✅ **Logs aggregated by Loki** (if configured)
- ✅ **Traces** (if you add instrumentation)
- ✅ **Dashboards in Grafana**

Add Prometheus annotations to your deployment:
```yaml
  template:
    metadata:
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8080"
        prometheus.io/path: "/metrics"
```

---

## Rollback Plan

If issues arise, you can quickly rollback to VM:

1. Keep VM running during initial migration
2. Test thoroughly in Kubernetes first
3. Only decommission VM after 2+ weeks of stable operation

---

## Comparison: VM vs Kubernetes

| Feature | VM (Current) | Kubernetes (New) |
|---------|-------------|------------------|
| **High Availability** | Single instance | Multiple replicas across zones |
| **Scaling** | Manual VM resize | Automatic horizontal scaling |
| **Deployments** | SSH + Docker commands | GitOps (git push = deploy) |
| **Rollbacks** | Manual docker compose | One command (`kubectl rollout undo`) |
| **Monitoring** | Manual setup | Automatic (Prometheus/Grafana) |
| **Backups** | Manual scripts | Automatic (Velero) |
| **Cost** | Pay for VM 24/7 | Pay per pod (can scale to zero) |
| **Access** | IAP tunnel to VM | Port-forward or Ingress |

---

## Support & Questions

Contact the Aurora Platform team for:
- Creating ArgoCD Application
- Namespace creation
- Secrets management setup
- Ingress/DNS configuration
- Troubleshooting deployment issues

**Platform Team:** [Your contact info]

---

## Quick Reference Commands

### Check Application Status
```bash
# See ArgoCD sync status
kubectl get application security-converter -n aurora-platform

# Check pods
kubectl get pods -n security-converter

# View logs
kubectl logs -n security-converter -l app=security-converter,component=api

# Describe pod for issues
kubectl describe pod <pod-name> -n security-converter
```

### Access Application
```bash
# Port forward to API
kubectl port-forward -n security-converter svc/security-converter-api 8080:80

# Port forward to database (for debugging)
kubectl port-forward -n security-converter svc/security-converter-db 5432:5432
```

### Force Sync in ArgoCD
```bash
kubectl patch application security-converter -n aurora-platform \
  --type merge \
  -p '{"operation":{"initiatedBy":{"username":"admin"},"sync":{}}}'
```

---

**Next Steps:**
1. Development team creates Kubernetes manifests
2. Push manifests to Git repository
3. Contact platform team to create ArgoCD Application
4. Platform team deploys and verifies
5. Migrate database data
6. Test thoroughly
7. Switch traffic (update documentation/links)
8. Decommission VM after stability period

Good luck with the migration! 🚀
