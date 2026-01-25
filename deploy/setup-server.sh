#!/bin/bash
# Server setup script for nebula deployment
# Run as root on the VPS server

set -e

echo "=== Nebula Server Setup ==="

# Install nginx if not present
if ! command -v nginx &> /dev/null; then
    echo "Installing nginx..."
    apt-get update
    apt-get install -y nginx
fi

# Create app directory
echo "Creating /opt/nebula..."
mkdir -p /opt/nebula
cd /opt/nebula

# Create .env file (edit values!)
if [ ! -f .env ]; then
    echo "Creating .env file..."
    cat > .env << 'EOF'
DB_PASSWORD=CHANGE_ME_STRONG_PASSWORD
RESEND_API_KEY=re_YOUR_API_KEY
TURNSTILE_SITE_KEY=YOUR_SITE_KEY
TURNSTILE_SECRET_KEY=YOUR_SECRET_KEY
EOF
    echo "!!! IMPORTANT: Edit /opt/nebula/.env with real values !!!"
fi

# Copy docker-compose
echo "Copy docker-compose.prod.yml to /opt/nebula/"

# Copy nginx config
echo "Setting up nginx..."
cp deploy/nginx-alnovis.io.conf /etc/nginx/sites-available/alnovis.io
ln -sf /etc/nginx/sites-available/alnovis.io /etc/nginx/sites-enabled/
nginx -t && systemctl reload nginx

# Create content directory
mkdir -p /opt/nebula/content/blog /opt/nebula/content/projects

echo ""
echo "=== Setup Complete ==="
echo ""
echo "Next steps:"
echo "1. Edit /opt/nebula/.env with real values"
echo "2. Copy content files to /opt/nebula/content/"
echo "3. Copy docker-compose.prod.yml to /opt/nebula/"
echo "4. Run: cd /opt/nebula && docker compose -f docker-compose.prod.yml up -d"
echo ""
echo "GitHub Actions secrets needed:"
echo "  VPS_HOST: 64.226.96.23"
echo "  VPS_USER: your_username"
echo "  VPS_SSH_KEY: your_private_key"
