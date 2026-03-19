# SSL/TLS Certificates for EMQX

This directory contains SSL/TLS certificates for secure MQTT connections.

## Required Files

Copy your certificate files here:

- `broker.i-ot.net.crt` - SSL certificate file
- `broker.i-ot.net.key` - SSL private key file

## Tunneling Server Configuration

The production tunneling server uses certificates located at:
- `/etc/emqx/certs/broker.i-ot.net.crt`
- `/etc/emqx/certs/broker.i-ot.net.key`

## Usage

### Development (docker-compose-dev.yml)
```bash
# Place your certs in this directory, then run:
make dev
```

### Production (docker-compose.yml)
```bash
# Ensure certs are in this directory, then run:
make prod
```

## Certificate Format

EMQX expects PEM-encoded certificates:
```
-----BEGIN CERTIFICATE-----
...
-----END CERTIFICATE-----
```

```
-----BEGIN PRIVATE KEY-----
...
-----END PRIVATE KEY-----
```

## Security Note

⚠️ **Never commit private keys to version control!**

This directory is gitignored by default. Ensure your `.key` files remain private.
