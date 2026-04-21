# SSL/TLS Certificates for EMQX

This directory contains SSL/TLS certificates for secure MQTT connections.

## Required Files

Place your certificate files in this directory:

- `broker.i-ot.net.crt` - SSL certificate file
- `broker.i-ot.net.key` - SSL private key file

## Usage

### Development

For local development with Docker Compose:

```bash
# Place your certs in this directory, then run:
make dev
```

The certificates will be mounted to the EMQX container at:
- `/etc/emqx/certs/broker.i-ot.net.crt`
- `/etc/emqx/certs/broker.i-ot.net.key`

### Production

For production deployments:

1. **Obtain valid certificates** from a trusted CA (e.g., Let's Encrypt, DigiCert)
2. **Place certificates** in a secure location on your production server
3. **Configure Docker Compose** or Kubernetes to mount certificates
4. **Set appropriate permissions** - private key should be readable only by EMQX

Example Docker Compose volume mount:
```yaml
volumes:
  - /path/to/certs/broker.i-ot.net.crt:/etc/emqx/certs/broker.i-ot.net.crt:ro
  - /path/to/certs/broker.i-ot.net.key:/etc/emqx/certs/broker.i-ot.net.key:ro
```

## Certificate Format

EMQX expects PEM-encoded certificates:

**Certificate:**
```
-----BEGIN CERTIFICATE-----
MIIDXTCCAkWgAwIBAgIJAJC1HiIAZAiUMA0Gcg...
...
-----END CERTIFICATE-----
```

**Private Key:**
```
-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSk...
...
-----END PRIVATE KEY-----
```

## Certificate Requirements

- **Key Size:** Minimum 2048-bit RSA (4096-bit recommended)
- **Signature Algorithm:** SHA-256 or better
- **Validity:** Appropriate for your deployment (1 year typical for Let's Encrypt)
- **Common Name (CN):** Should match your MQTT broker hostname
- **Subject Alternative Names (SAN):** Include all hostnames/IPs clients will use

## Testing Certificates

Verify your certificates:

```bash
# Check certificate details
openssl x509 -in broker.i-ot.net.crt -text -noout

# Verify certificate and key match
openssl x509 -noout -modulus -in broker.i-ot.net.crt | openssl md5
openssl rsa -noout -modulus -in broker.i-ot.net.key | openssl md5
# Both commands should output the same MD5 hash

# Test certificate expiration
openssl x509 -noout -dates -in broker.i-ot.net.crt
```

## Let's Encrypt (Recommended for Production)

Obtain free, trusted certificates using Let's Encrypt:

```bash
# Using certbot
certbot certonly --standalone -d broker.i-ot.net

# Certificates will be placed in:
# /etc/letsencrypt/live/broker.i-ot.net/fullchain.pem
# /etc/letsencrypt/live/broker.i-ot.net/privkey.pem

# Copy or symlink to this directory
cp /etc/letsencrypt/live/broker.i-ot.net/fullchain.pem broker.i-ot.net.crt
cp /etc/letsencrypt/live/broker.i-ot.net/privkey.pem broker.i-ot.net.key
```

## Self-Signed Certificates (Development Only)

For development/testing only:

```bash
# Generate private key
openssl genrsa -out broker.i-ot.net.key 4096

# Generate certificate
openssl req -new -x509 -days 365 \
  -key broker.i-ot.net.key \
  -out broker.i-ot.net.crt \
  -subj "/CN=broker.i-ot.net"
```

⚠️ **Warning:** Self-signed certificates will cause certificate warnings in clients and should only be used for development.

## Security Notes

⚠️ **Never commit private keys to version control!**

This directory is gitignored by default. Ensure your `.key` files remain private.

### Best Practices

1. **Restrict file permissions:**
   ```bash
   chmod 644 broker.i-ot.net.crt
   chmod 600 broker.i-ot.net.key
   ```

2. **Regular rotation:** Renew certificates before expiration

3. **Monitor expiration:** Set up alerts for certificate expiry

4. **Backup securely:** Store certificate backups in encrypted storage

5. **Use trusted CAs:** For production, always use certificates from trusted Certificate Authorities

## Troubleshooting

### Certificate Verification Failed

- Ensure certificate chain is complete (include intermediate certificates)
- Verify certificate is not expired
- Check that CN/SAN matches the hostname clients are using

### Private Key Issues

- Verify key file is not corrupted
- Ensure key matches certificate (use modulus check above)
- Check file permissions allow EMQX to read the key

### Connection Errors

- Verify EMQX configuration points to correct certificate paths
- Check that certificate files are mounted correctly in Docker
- Review EMQX logs for SSL/TLS error messages

## Additional Resources

- [EMQX SSL/TLS Documentation](https://www.emqx.io/docs/en/v5.0/deploy/ssl.html)
- [Let's Encrypt Documentation](https://letsencrypt.org/docs/)
- [OpenSSL Certificate Management](https://www.openssl.org/docs/man1.1.1/man1/x509.html)
