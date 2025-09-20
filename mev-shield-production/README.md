# MEV Shield Production Deployment

This package contains everything needed to deploy MEV Shield in production.

## Prerequisites

- Docker 20.10+
- Docker Compose 2.0+
- 4GB+ RAM
- 20GB+ disk space

## Quick Start

1. Extract this package to your server
2. Run the deployment script:
   ```bash
   chmod +x deploy.sh
   ./deploy.sh
   ```

## Services

- **Frontend**: React application served by Nginx
- **Backend Mock**: Node.js API server (port 8080)
- **Backend Live**: Real-time MEV protection (port 8096)
- **Nginx**: Reverse proxy with SSL termination

## URLs

- Main Application: https://localhost
- Admin Dashboard: https://localhost/admin
- User Dashboard: https://localhost/dashboard
- Builder Dashboard: https://localhost/builder
- Trader Dashboard: https://localhost/trader

## API Endpoints

- Mock API: https://localhost/api/
- Live Protection: https://localhost/api/live/
- WebSocket: wss://localhost/ws/

## Management Commands

```bash
# View logs
docker-compose logs -f

# Stop all services
docker-compose down

# Restart specific service
docker-compose restart [service-name]

# Update and restart
git pull  # if using git
docker-compose up --build -d
```

## SSL Certificates

The deployment script generates self-signed certificates. For production:

1. Replace `ssl/privkey.pem` and `ssl/fullchain.pem` with real certificates
2. Restart nginx: `docker-compose restart nginx`

## Test Accounts

- admin@mevshield.ai / admin123
- user@mevshield.ai / user123
- builder@mevshield.ai / builder123
- trader@mevshield.ai / trader123

## Troubleshooting

1. Check container status: `docker-compose ps`
2. View specific logs: `docker-compose logs [service-name]`
3. Check ports: `netstat -tlnp | grep -E '(80|443|8080|8096)'`
4. Restart all: `docker-compose down && docker-compose up -d`
