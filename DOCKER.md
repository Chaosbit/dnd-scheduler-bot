# Docker Deployment Guide

This guide explains how to deploy the D&D Scheduler Bot using Docker and Docker Compose.

## Prerequisites

- Docker Engine 20.10+
- Docker Compose v2.0+
- A Telegram Bot Token (obtained from [@BotFather](https://t.me/BotFather))

## Quick Start

1. **Clone the repository:**
   ```bash
   git clone <repository-url>
   cd dnd-scheduler-bot
   ```

2. **Set up environment variables:**
   ```bash
   cp .env.example .env
   # Edit .env with your bot token
   export TELEGRAM_BOT_TOKEN="your_bot_token_here"
   ```

3. **Start the application:**
   ```bash
   docker-compose up -d
   ```

4. **Check the health:**
   ```bash
   curl http://localhost:3000/health
   ```

## Configuration

### Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `TELEGRAM_BOT_TOKEN` | Your Telegram bot token | - | Yes |
| `DATABASE_URL` | SQLite database path | `sqlite:/app/data/scheduler.db` | No |
| `HTTP_PORT` | Health check server port | `3000` | No |
| `RUST_LOG` | Logging level | `info` | No |

### Docker Compose Profiles

The docker-compose.yml includes different profiles for different deployment scenarios:

#### Development (Default)
```bash
docker-compose up -d
```
Starts just the bot with health monitoring.

#### Production with Nginx
```bash
docker-compose --profile production up -d
```
Includes an Nginx reverse proxy for the health endpoints with:
- Rate limiting
- Security headers
- Custom error pages
- SSL-ready configuration

## Health Monitoring

The application exposes several health check endpoints:

- `/health` - Comprehensive health check with database status
- `/health/ready` - Readiness probe (database connectivity)
- `/health/live` - Liveness probe (simple alive check)

### Example Health Response
```json
{
  "status": "healthy",
  "timestamp": "2024-11-27T10:30:00Z",
  "version": "0.1.0",
  "database": {
    "status": "healthy",
    "connection_pool_size": 10,
    "response_time_ms": 5
  },
  "uptime_seconds": 3600
}
```

## Data Persistence

The application uses a named Docker volume `scheduler_data` to persist:
- SQLite database files
- Application data

### Backup Data
```bash
# Create a backup
docker run --rm -v scheduler_data:/data -v $(pwd):/backup alpine tar czf /backup/scheduler-backup-$(date +%Y%m%d).tar.gz -C /data .

# Restore from backup
docker run --rm -v scheduler_data:/data -v $(pwd):/backup alpine tar xzf /backup/scheduler-backup-YYYYMMDD.tar.gz -C /data
```

## Database Migrations

Database migrations are automatically run when the container starts. The init container ensures proper database setup before the main application starts.

### Manual Migration
If you need to run migrations manually:
```bash
docker-compose exec dnd-scheduler /app/migrate up
```

## Monitoring and Logs

### View Logs
```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f dnd-scheduler

# Last 100 lines
docker-compose logs --tail 100 dnd-scheduler
```

### Container Status
```bash
docker-compose ps
```

### Health Check Status
```bash
# Docker health status
docker inspect dnd-scheduler-bot | grep -A 10 "Health"

# Application health endpoint
curl -s http://localhost:3000/health | jq
```

## Security Considerations

### Container Security
- Runs as non-root user (`appuser`)
- Uses minimal Alpine Linux base image
- Includes security headers in Nginx configuration
- Database files are not exposed outside the container

### Network Security
- Uses isolated Docker network
- Health endpoints can be protected with Nginx rate limiting
- Telegram bot token is passed via environment variables

## Troubleshooting

### Common Issues

1. **Bot not responding to commands:**
   - Check bot token is correct: `docker-compose logs dnd-scheduler | grep "token"`
   - Verify bot is started: `curl http://localhost:3000/health`

2. **Database errors:**
   - Check database file permissions
   - Verify migrations ran successfully: `docker-compose logs scheduler_init`

3. **Health check failures:**
   - Check if port 3000 is accessible: `docker-compose port dnd-scheduler 3000`
   - Verify health endpoint: `curl -v http://localhost:3000/health/live`

### Debug Mode
Run with debug logging:
```bash
RUST_LOG=debug docker-compose up
```

### Container Shell Access
```bash
# Get shell in running container
docker-compose exec dnd-scheduler sh

# Run one-off container for debugging
docker-compose run --rm dnd-scheduler sh
```

## Production Deployment

For production deployment:

1. **Use the production profile:**
   ```bash
   docker-compose --profile production up -d
   ```

2. **Set up SSL with Let's Encrypt:**
   - Configure your domain to point to the server
   - Use certbot or similar to obtain SSL certificates
   - Mount certificates in the Nginx container

3. **Configure monitoring:**
   - Set up log aggregation (ELK stack, Grafana)
   - Monitor health endpoints
   - Set up alerts for container failures

4. **Regular backups:**
   - Automate database backups
   - Store backups in secure, off-site location
   - Test restore procedures regularly

## Updates and Maintenance

### Update the Application
```bash
# Pull latest changes
git pull

# Rebuild and restart
docker-compose build --no-cache
docker-compose up -d
```

### Clean Up
```bash
# Remove old images
docker image prune

# Remove unused volumes (be careful!)
docker volume prune
```

## Support

For issues and questions:
- Check the logs first: `docker-compose logs`
- Verify health endpoints are responding
- Check GitHub issues for known problems