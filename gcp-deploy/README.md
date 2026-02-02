# GCP Deployment Guide (No Public IP)

This guide deploys the Security Classification Converter to a GCP e2-medium instance without a public IP, using IAP (Identity-Aware Proxy) for secure access.

## Prerequisites

1. **gcloud CLI** installed and authenticated
2. **IAP API** enabled in your project
3. **Firewall rule** allowing IAP to reach your VM

## Step 1: Enable Required APIs

```bash
export PROJECT_ID="sandbox-caf-compute-hub"
gcloud config set project $PROJECT_ID

# Enable required APIs
gcloud services enable compute.googleapis.com
gcloud services enable iap.googleapis.com
```

## Step 2: Create Firewall Rule for IAP

```bash
# Allow IAP to connect to your VM (IAP's IP range: 35.235.240.0/20)
gcloud compute firewall-rules create allow-iap-ssh \
    --direction=INGRESS \
    --priority=1000 \
    --network=default \
    --action=ALLOW \
    --rules=tcp:22,tcp:8080 \
    --source-ranges=35.235.240.0/20 \
    --target-tags=iap-enabled
```

## Step 3: Create the VM

```bash
export ZONE="northamerica-northeast1-a"  # Change to your preferred zone
export VM_NAME="security-converter-vm"

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
```

## Step 4: Grant IAP Access

```bash
# Grant yourself IAP tunnel access
gcloud projects add-iam-policy-binding $PROJECT_ID \
    --member="user:YOUR_EMAIL@domain.com" \
    --role="roles/iap.tunnelResourceAccessor"
```

## Step 5: Connect to the VM via IAP

```bash
# SSH into the VM
gcloud compute ssh $VM_NAME --zone=$ZONE --tunnel-through-iap

# Or start an IAP tunnel for port forwarding
gcloud compute start-iap-tunnel $VM_NAME 8080 --local-host-port=localhost:8080 --zone=$ZONE
```

## Step 6: Deploy the Application

Once connected via SSH:

```bash
# Check startup script completed
sudo tail -f /var/log/startup-script.log

# Navigate to app directory
cd /opt/security-converter

# Update the .env file with secure values
sudo nano .env

# Option A: Pull pre-built image (if you push to a registry)
# sudo docker compose pull
# sudo docker compose up -d

# Option B: Build locally (copy source files first)
# See "Copying Source Files" section below
```

## Copying Source Files to VM

Since there's no public IP, use gcloud SCP through IAP:

```bash
# From your local machine, copy the project files
cd x:\ravi\dev\security_converter

# Create a tarball of necessary files
tar -czvf security-converter.tar.gz \
    Cargo.toml Cargo.lock \
    graphql_api errors migrations templates \
    Dockerfile.slim docker-compose.yml .env

# Copy to VM via IAP
gcloud compute scp security-converter.tar.gz $VM_NAME:/tmp/ --zone=$ZONE --tunnel-through-iap

# SSH in and extract
gcloud compute ssh $VM_NAME --zone=$ZONE --tunnel-through-iap

# On the VM:
cd /opt/security-converter
sudo tar -xzvf /tmp/security-converter.tar.gz
sudo docker compose up -d --build
```

## Step 7: Access the Application

Start an IAP tunnel to access the app locally:

```bash
# In a terminal, start the tunnel (keeps running)
gcloud compute start-iap-tunnel $VM_NAME 8080 --local-host-port=localhost:8080 --zone=$ZONE
```

Then open http://localhost:8080/playground in your browser.

## Useful Commands

```bash
# Check container status
sudo docker compose ps

# View logs
sudo docker compose logs -f api

# Restart services
sudo docker compose restart

# Stop services
sudo docker compose down

# Update and rebuild
sudo docker compose down
sudo docker compose up -d --build
```

## Security Notes

1. **Change default passwords** in `.env` before deploying
2. **Database credentials** should use strong passwords in production
3. **Consider Cloud SQL** for managed PostgreSQL with automatic backups
4. **Set up Cloud Armor** if you add a load balancer later

## Troubleshooting

### Can't connect via IAP
- Ensure IAP API is enabled
- Check firewall rule exists and has correct tags
- Verify IAM permissions for `roles/iap.tunnelResourceAccessor`

### Container won't start
```bash
sudo docker compose logs
sudo journalctl -u docker
```

### Database connection issues
```bash
# Check if postgres is running
sudo docker compose ps
sudo docker compose logs db
```
