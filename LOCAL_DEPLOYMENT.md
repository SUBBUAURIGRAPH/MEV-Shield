# 🚀 MEV Shield Local Deployment Guide

## Quick Start

The fastest way to get MEV Shield running locally:

```bash
# Install dependencies and deploy everything
make quick-start

# Or manually:
./build-and-deploy-local.sh
```

## Prerequisites

### Required Software
- **Docker** 20.10+ and **Docker Compose** 2.0+
- **Node.js** 18+ and **npm** 8+
- **Rust** 1.75+ (optional - handled by Docker)

### Installation Commands
```bash
# macOS (using Homebrew)
brew install docker docker-compose node

# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Deployment Options

### Option 1: Quick Deploy (Recommended)
```bash
make deploy
```

### Option 2: Development Environment
```bash
# Start only database and cache services
make dev

# Then run services manually:
cargo run                    # Terminal 1: Rust backend
cd dashboard && npm start    # Terminal 2: React frontend
```

### Option 3: Manual Build and Deploy
```bash
# Build everything
make build

# Deploy with Docker Compose
docker-compose -f docker-compose.local.yml up -d
```

## Services and Ports

Once deployed, MEV Shield services are available at:

| Service | URL | Purpose |
|---------|-----|---------|
| **User Dashboard** | http://localhost:3002 | Main user interface |
| **Admin Dashboard** | http://localhost:3001 | Admin control panel |
| **Core API** | http://localhost:8080 | REST API endpoints |
| **Nginx Proxy** | http://localhost | Load balancer |
| **Grafana** | http://localhost:3000 | Monitoring (admin/admin) |
| **Prometheus** | http://localhost:9091 | Metrics collection |

### Database Services
| Service | Connection | Credentials |
|---------|------------|-------------|
| **PostgreSQL** | localhost:5432 | mev_user/mev_password |
| **Redis** | localhost:6379 | No auth |

## Service Management

### Start/Stop Services
```bash
make deploy          # Start all services
make stop           # Stop all services
make restart        # Restart all services
```

### View Logs
```bash
make logs           # All service logs
make logs-core      # MEV Shield core only
make logs-admin     # Admin dashboard only
make logs-user      # User dashboard only
```

### Service Status
```bash
make status         # Service status
make health         # Health checks
```

## Development Commands

### Building
```bash
make build          # Build Rust + React
make build-docker   # Build Docker images
```

### Testing
```bash
make test           # Run all tests
make test-rust      # Rust tests only
make test-js        # React tests only
```

### Code Quality
```bash
make lint           # Run linters
make format         # Format code
make check          # Lint + test
```

### Database Management
```bash
make db-migrate     # Run migrations
make db-reset       # Reset database
make backup         # Backup database
make restore BACKUP=filename  # Restore from backup
```

## Directory Structure

```
MEV-Shield/
├── src/                     # Rust backend source
├── dashboard/               # React frontend
├── docker-compose.local.yml # Local Docker setup
├── nginx/                   # Nginx configuration
├── monitoring/              # Prometheus/Grafana config
├── build-and-deploy-local.sh # Main deployment script
├── Makefile                 # Development commands
└── logs/                    # Application logs
```

## Configuration

### Environment Variables
Key environment variables for local deployment:

```bash
# Database
DATABASE_URL=postgresql://mev_user:mev_password@localhost:5432/mev_shield
REDIS_URL=redis://localhost:6379

# Application
MEV_SHIELD_ENV=local
RUST_LOG=debug

# Frontend
REACT_APP_API_URL=http://localhost:8080
REACT_APP_ENV=local
```

### Config Files
- `config.toml` - Main application configuration
- `docker-compose.local.yml` - Docker services
- `nginx/nginx.conf` - Nginx routing
- `monitoring/prometheus.yml` - Metrics configuration

## Troubleshooting

### Common Issues

#### Port Conflicts
```bash
# Check what's using a port
lsof -i :3000
lsof -i :8080

# Kill process using port
kill -9 <PID>
```

#### Docker Issues
```bash
# Clean Docker resources
make clean-docker

# View Docker logs
docker-compose -f docker-compose.local.yml logs <service-name>

# Restart specific service
docker-compose -f docker-compose.local.yml restart <service-name>
```

#### Build Failures
```bash
# Clean build artifacts
make clean

# Rebuild from scratch
make clean && make build
```

#### Database Connection Issues
```bash
# Check PostgreSQL status
docker-compose -f docker-compose.local.yml exec postgres pg_isready

# Connect to database manually
make shell-db

# Reset database
make db-reset
```

### Health Checks

Check if all services are healthy:

```bash
curl http://localhost:8080/health    # Core API
curl http://localhost:3001           # Admin Dashboard
curl http://localhost:3002           # User Dashboard
```

### Logs and Debugging

```bash
# Application logs
tail -f logs/deploy.log

# Docker container logs
docker-compose -f docker-compose.local.yml logs -f

# Access container shell
make shell
```

## Performance Tuning

### Resource Allocation
Adjust Docker resource limits in `docker-compose.local.yml`:

```yaml
services:
  mev-shield-core:
    deploy:
      resources:
        limits:
          memory: 1G
          cpus: '0.5'
```

### Database Optimization
```bash
# Increase PostgreSQL connections
docker-compose -f docker-compose.local.yml exec postgres \
  psql -U mev_user -d mev_shield -c "ALTER SYSTEM SET max_connections = 200;"
```

## Security Considerations

### Local Development Only
⚠️ **This setup is for local development only!**

- Default passwords are used
- No SSL/TLS encryption
- Debug logging enabled
- No rate limiting

### Production Deployment
For production deployment, see:
- Use strong passwords
- Enable SSL/TLS
- Configure firewalls
- Set up monitoring alerts
- Use secrets management

## Advanced Usage

### Custom Configuration
```bash
# Use custom config file
docker-compose -f docker-compose.local.yml -f docker-compose.custom.yml up -d

# Override environment variables
MEV_SHIELD_ENV=development make deploy
```

### Development Workflows
```bash
# Hot reload development
make dev
# Then run `cargo watch -x run` and `npm start` in separate terminals

# Test specific components
make test-rust
cd dashboard && npm test -- --testNamePattern="Admin"
```

### Monitoring and Metrics
- **Grafana Dashboard**: http://localhost:3000
  - Username: admin
  - Password: admin
  - Pre-configured MEV Shield metrics

- **Prometheus Metrics**: http://localhost:9091
  - Application metrics
  - System metrics
  - Custom MEV protection metrics

## Getting Help

### Documentation
- [README.md](./README.md) - Project overview
- [BUILD_AND_RUN.md](./BUILD_AND_RUN.md) - Detailed build instructions
- [COMPETITORS_ANALYSIS.md](./COMPETITORS_ANALYSIS.md) - Market analysis

### Commands Reference
```bash
make help           # Show all available commands
./build-and-deploy-local.sh --help  # Deployment script help
```

### Logs and Debugging
- Check `logs/deploy.log` for deployment issues
- Use `make logs` to view service logs
- Enable debug logging with `RUST_LOG=debug`

---

🎉 **You're ready to go!** Visit http://localhost:3002 to start using MEV Shield locally.