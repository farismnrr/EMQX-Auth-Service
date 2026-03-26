# EMQX Auth Service Production Setup

This document describes how to configure EMQX Auth Service and EMQX Broker (v5+) for production with AES-256-GCM and JWT.

## Listener Baseline

For production deployments, configure EMQX listeners explicitly and expose only required ports.

## Certificates

Ensure SSL certificate files are available at:

`/etc/emqx/certs/`

Use valid server certificates and private keys signed by a trusted CA (or your internal CA).

## Auth Service Endpoints

The service provides three EMQX-facing endpoints:

1. Credential authentication endpoint
2. JWT issuance endpoint
3. ACL authorization endpoint (publish/subscribe checks)

## Authentication Logic

- EMQX sends username/password to the auth endpoint.
- Service decrypts stored ciphertext (AES key from environment).
- Service compares credentials and returns allow/deny response.

## JWT Flow

- Client sends credentials to token endpoint.
- Service validates credentials and issues a signed JWT.
- Signing key is read from `SECRET_KEY`.

## ACL Logic

- Superuser: full topic access.
- Regular user: topic access constrained by username prefix policy.

## Required Environment Variables

```env
SECRET_KEY=replace-with-strong-secret
MQTT_PASS_ENCRYPTION_KEY=64-hex-char-key
EMQX_API_KEY=replace-with-internal-api-key
```

## Production Hardening

1. Bind auth service to internal network (`127.0.0.1` or private Docker network).
2. Do not expose internal auth ports publicly.
3. Rotate encryption and signing keys with a controlled plan.
4. Monitor server-side logs for decryption/auth failures.
5. Restrict superuser usage to backend/admin accounts only.

## EMQX Config Recommendations

- Prefer HOCON configuration in production.
- Set deny-by-default behavior for unmatched authorization rules.
- Keep auth and ACL hooks deterministic and observable.

## Validation Checklist

- [ ] TLS listener active with valid cert chain
- [ ] Auth endpoint reachable from EMQX
- [ ] JWT endpoint signs and verifies tokens correctly
- [ ] ACL endpoint enforces topic boundaries
- [ ] Internal API keys and secrets are not logged

## Incident Response Notes

If encryption keys are leaked:

1. Rotate keys immediately.
2. Re-encrypt stored credentials as required.
3. Invalidate issued tokens if signing key was affected.
4. Review logs and access history.
