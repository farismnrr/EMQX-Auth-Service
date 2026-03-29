# EMQX Auth Service Production Setup

This document describes how to configure and deploy EMQX Auth Service for production environments.

## Overview

EMQX Auth Service provides REST API-based user management and JWT token generation for MQTT client authentication. The service uses PostgreSQL for data persistence and OpenTelemetry for observability.

## Architecture

```
┌─────────────────┐      REST API        ┌──────────────────────┐
│  EMQX Broker    │ ◄──────────────────► │  EMQX Auth Service   │
│  (JWT Auth)     │    JWT Token         │  (Port 5500)         │
└─────────────────┘                      └──────────────────────┘
                                                │
                                                ▼
                                         ┌─────────────────┐
                                         │   PostgreSQL    │
                                         │   Database      │
                                         └─────────────────┘
```

## Authentication Flow

### Device Connection Flow

1. **Device requests JWT token** from backend or directly from auth service
2. **Auth service validates** device exists in database
3. **Auth service issues JWT** token (24-hour expiration)
4. **Device connects to EMQX** using JWT token as password
5. **EMQX validates JWT** using configured secret key

### Backend User Management

Backend systems manage MQTT users via REST API:
- `POST /mqtt/create` - Create new MQTT user
- `GET /mqtt` - List all MQTT users
- `DELETE /mqtt/{username}` - Remove MQTT user
- `POST /mqtt/jwt` - Generate JWT token for user

## Required Environment Variables

### Database Configuration

```env
# Option 1: Connection string (recommended)
DATABASE_URL=postgresql://user:password@host:5432/emqx_auth

# Option 2: Individual parameters
DB_HOST=localhost
DB_PORT=5432
DB_NAME=emqx_auth
DB_USER=postgres
DB_PASSWORD=your_secure_password
DB_SCHEMA=public

# Optional: Connection pool settings
DB_MAX_CONNECTIONS=10
DB_CONNECT_TIMEOUT=5000
DB_IDLE_TIMEOUT=60000
```

### Authentication & Security

```env
# Required: Secret key for JWT signing (SHA256 hash)
# Generate with: openssl rand -hex 32
SECRET_KEY=<64-character-hex-string>

# Required: API key for REST endpoint authentication
# Generate with: openssl rand -hex 32
API_KEY=<64-character-hex-string>
```

### Logging & Performance

```env
# Optional: Log level (default: info)
RUST_LOG=info

# Optional: Rate limit for MQTT auth operations (default: 100 rpm)
MQTT_AUTH_RATE_LIMIT=100
```

### OpenTelemetry (Optional)

```env
# OTLP endpoint for traces and metrics
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317

# Service name for telemetry
OTEL_SERVICE_NAME=emqx-auth-service
```

## Production Deployment

### Docker Compose Example

```yaml
version: '3.8'

services:
  emqx-auth-service:
    image: ghcr.io/farismnrr/emqx-auth-service:latest
    ports:
      - "5500:5500"
    environment:
      - DATABASE_URL=postgresql://user:pass@postgres:5432/emqx_auth
      - SECRET_KEY=${SECRET_KEY}
      - API_KEY=${API_KEY}
      - RUST_LOG=info
      - OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317
      - OTEL_SERVICE_NAME=emqx-auth-service
    depends_on:
      - postgres
    networks:
      - internal
    restart: unless-stopped

  postgres:
    image: postgres:15-alpine
    environment:
      - POSTGRES_DB=emqx_auth
      - POSTGRES_USER=user
      - POSTGRES_PASSWORD=pass
    volumes:
      - postgres_data:/var/lib/postgresql/data
    networks:
      - internal
    restart: unless-stopped

volumes:
  postgres_data:

networks:
  internal:
    driver: bridge
```

### EMQX Configuration

Configure EMQX to use JWT authentication:

```hocon
# /etc/emqx/emqx.conf
authentication {
  backend = "jwt"
  jwt {
    algorithm = "hs256"
    secret = "${SECRET_KEY}"  # Same as auth service
    from = "password"
  }
}
```

## Production Hardening

### Network Security

1. **Bind to internal network only** - Do not expose auth service publicly
2. **Use private Docker network** - Isolate database and auth service
3. **Restrict API access** - Only backend services should access REST API
4. **Enable TLS** - Use HTTPS for all REST API communication

### Secret Management

1. **Use environment variables** - Never hardcode secrets in config files
2. **Rotate keys regularly** - Establish key rotation schedule
3. **Use secret manager** - Consider HashiCorp Vault, AWS Secrets Manager
4. **Restrict access** - Limit who can access SECRET_KEY and API_KEY

### Database Security

1. **Use strong passwords** - Generate secure database credentials
2. **Restrict permissions** - Database user should only access emqx_auth schema
3. **Enable SSL** - Use encrypted database connections
4. **Regular backups** - Implement automated backup strategy

### Monitoring & Observability

1. **Enable OpenTelemetry** - Export traces and metrics
2. **Monitor error rates** - Alert on authentication failures
3. **Track response times** - Monitor API latency
4. **Log aggregation** - Centralize logs for analysis

## Validation Checklist

Before going to production:

- [ ] Database connection established and tested
- [ ] SECRET_KEY generated and configured (64 hex chars)
- [ ] API_KEY generated and configured (64 hex chars)
- [ ] Health check endpoint responds: `GET /`
- [ ] User creation works: `POST /mqtt/create`
- [ ] JWT generation works: `POST /mqtt/jwt`
- [ ] EMQX accepts JWT tokens for MQTT connection
- [ ] OpenTelemetry traces visible in backend
- [ ] API endpoints not publicly accessible
- [ ] Database backups configured
- [ ] Monitoring and alerting configured

## Incident Response

### JWT Token Issues

If devices cannot connect:
1. Check JWT token expiration (24 hours)
2. Verify SECRET_KEY matches EMQX configuration
3. Check auth service logs for token generation errors
4. Test JWT endpoint manually: `POST /mqtt/jwt`

### Database Connection Failures

If auth service cannot connect to database:
1. Check DATABASE_URL format and credentials
2. Verify PostgreSQL is running and accessible
3. Check network connectivity between containers
4. Review database logs for connection errors

### Key Rotation Procedure

To rotate SECRET_KEY:
1. Generate new SECRET_KEY
2. Update EMQX configuration first
3. Restart EMQX broker
4. Update auth service configuration
5. Restart auth service
6. All existing JWT tokens will be invalidated

To rotate API_KEY:
1. Generate new API_KEY
2. Update all backend services using the API
3. Update auth service configuration
4. Restart auth service
5. Test all API endpoints

## Performance Tuning

### Connection Pool

Adjust database connection pool based on load:
```env
DB_MAX_CONNECTIONS=20        # Increase for high load
DB_CONNECT_TIMEOUT=10000     # Increase for slow networks
DB_IDLE_TIMEOUT=120000       # Increase for bursty traffic
```

### Rate Limiting

Adjust rate limit based on expected throughput:
```env
MQTT_AUTH_RATE_LIMIT=500     # Increase for high-volume deployments
```

## Support

For issues or questions:
- Check application logs: `docker logs <container>`
- Review OpenTelemetry traces
- Create an issue in the repository
