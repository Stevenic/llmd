# Deployment Guide for Acme Platform

This document describes how to deploy, configure, and operate the Acme Platform in production environments. It covers infrastructure requirements, the deployment pipeline, configuration management, monitoring, and incident response procedures.

---

## Infrastructure Requirements

### Compute

The platform requires a minimum of three application nodes behind a load balancer. Each node should have at least 4 vCPUs and 16 GB of RAM. For high-traffic deployments exceeding 10,000 requests per second, increase to 8 vCPUs and 32 GB of RAM per node.

- Application nodes: 3 minimum, 6 recommended for high availability
- Load balancer: Layer 7 with health check support
- Worker nodes: 2 minimum for background job processing
- Scheduler node: 1 dedicated instance for cron and periodic tasks

### Storage

All persistent data is stored in PostgreSQL. File uploads go to S3-compatible object storage.

- PostgreSQL 15 or later with streaming replication enabled
- Primary database: 500 GB SSD minimum
- Read replicas: 2 minimum for production workloads
- Object storage: S3 or compatible (MinIO, GCS with S3 gateway)
- Redis 7 or later for caching and session storage
- Redis memory: 8 GB minimum, 16 GB recommended

### Networking

- Private subnet for all application and database nodes
- Public subnet only for the load balancer and bastion host
- VPN or private link for cross-region replication
- DNS: Route 53 or equivalent with health-check-based failover
- TLS 1.3 required on all external endpoints
- Internal traffic uses mTLS between services

---

## Deployment Pipeline

### Overview

The platform uses a continuous deployment pipeline that runs on every merge to the main branch. The pipeline builds a container image, runs the test suite, performs a canary deployment, and then promotes to full rollout.

### Steps to Deploy a New Release

1. Open a pull request against the main branch with your changes.
2. The CI system runs the full test suite including unit tests, integration tests, and end-to-end tests.
3. A reviewer approves the pull request and it is merged to main.
4. The pipeline builds a new Docker image tagged with the commit SHA.
5. The image is pushed to the container registry and scanned for vulnerabilities.
6. If the scan passes, the pipeline deploys the new image to the canary node which receives 5% of traffic.
7. The canary runs for 10 minutes while automated checks monitor error rates, latency, and resource usage.
8. If all health checks pass during the canary window, the pipeline promotes the image to all remaining nodes using a rolling update strategy.
9. Each node is drained of active connections before the new image is started.
10. After all nodes are updated, the pipeline runs a smoke test suite against the production endpoint.
11. If the smoke tests pass, the deployment is marked as successful and the commit SHA is recorded as the current release.
12. If any step fails, the pipeline automatically rolls back to the previous known-good image.

### Rollback Procedure

In the event of a failed deployment or a post-deployment incident, rollback can be triggered manually or automatically.

- Automatic rollback triggers if error rate exceeds 5% during canary or the first 15 minutes after full rollout.
- Manual rollback is performed by running `acme deploy rollback --to <commit-sha>` from the operations CLI.
- Rollback completes in under 3 minutes for a typical 6-node cluster.
- Database migrations are designed to be backward-compatible so rollbacks do not require schema changes.

---

## Configuration Management

### Environment Variables

All runtime configuration is provided through environment variables. No configuration files are baked into the container image.

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| DATABASE_URL | Yes | — | PostgreSQL connection string |
| REDIS_URL | Yes | — | Redis connection string |
| S3_BUCKET | Yes | — | Object storage bucket name |
| S3_REGION | Yes | — | Object storage region |
| SECRET_KEY | Yes | — | Application secret for signing tokens |
| LOG_LEVEL | No | info | Logging verbosity: debug, info, warn, error |
| WORKER_CONCURRENCY | No | 4 | Number of concurrent background workers |
| MAX_UPLOAD_SIZE_MB | No | 50 | Maximum file upload size in megabytes |
| RATE_LIMIT_PER_MIN | No | 600 | API rate limit per authenticated user |
| ENABLE_FEATURE_FLAGS | No | false | Enable the feature flag evaluation system |

### Secrets Management

Secrets are stored in AWS Secrets Manager or HashiCorp Vault. They are injected into the container at startup and never written to disk.

- Database credentials rotate every 30 days automatically.
- API signing keys use RSA 4096-bit keys.
- TLS certificates are managed by cert-manager with automatic renewal.
- Service account tokens expire after 24 hours and are refreshed automatically.

### Feature Flags

The platform supports feature flags for gradual rollouts and A/B testing.

- Flags are defined in the admin dashboard under Settings > Feature Flags.
- Each flag has a name, description, and targeting rules.
- Targeting can be based on user ID, organization, geographic region, or percentage rollout.
- Flag evaluation is cached in Redis with a 30-second TTL.
- Disabled flags return the default value with zero latency overhead.

---

## Monitoring and Observability

### Metrics

The platform exports Prometheus-compatible metrics on port 9090 at the `/metrics` endpoint.

- `http_requests_total` — Total HTTP requests by method, path, and status code
- `http_request_duration_seconds` — Request latency histogram with p50, p95, and p99 buckets
- `db_query_duration_seconds` — Database query latency by query type
- `db_connections_active` — Current number of active database connections
- `cache_hit_ratio` — Redis cache hit rate
- `worker_jobs_processed_total` — Background jobs completed by queue name
- `worker_jobs_failed_total` — Failed background jobs by queue and error type
- `s3_upload_duration_seconds` — Object storage upload latency
- `feature_flag_evaluations_total` — Flag evaluations by flag name and result

### Logging

All application logs are structured JSON written to stdout. The container runtime collects them and forwards to the centralized logging system.

- Log format: JSON with fields for timestamp, level, message, request_id, user_id, and trace_id
- Log aggregation: Elasticsearch or Loki
- Retention: 30 days for info and above, 7 days for debug
- Sensitive fields like passwords and tokens are automatically redacted before logging

### Alerting

Alerts are configured in Grafana and delivered through PagerDuty for critical issues and Slack for warnings.

| Alert | Condition | Severity | Response Time |
|-------|-----------|----------|---------------|
| High Error Rate | Error rate > 5% for 5 minutes | Critical | 5 minutes |
| High Latency | p99 latency > 2 seconds for 10 minutes | Warning | 30 minutes |
| Database Connection Pool Exhaustion | Active connections > 90% of pool size | Critical | 5 minutes |
| Disk Usage High | Disk usage > 85% on any node | Warning | 1 hour |
| Certificate Expiry | TLS cert expires within 7 days | Warning | 24 hours |
| Failed Deployments | Two consecutive deployment failures | Critical | 15 minutes |
| Worker Queue Backlog | Queue depth > 10,000 for 15 minutes | Warning | 30 minutes |

### Distributed Tracing

The platform uses OpenTelemetry for distributed tracing across all services.

- Traces propagate via the W3C Trace Context header.
- Sampling rate is 10% for normal traffic and 100% for error responses.
- Trace data is exported to Jaeger with a 48-hour retention window.
- Each trace includes spans for HTTP handlers, database queries, cache operations, and external API calls.

---

## Incident Response

### Severity Levels

- **SEV-1**: Complete service outage or data loss affecting all users. Requires immediate response from the on-call engineer and incident commander.
- **SEV-2**: Degraded service affecting a significant portion of users. Feature is unusable but workarounds exist. Response within 15 minutes.
- **SEV-3**: Minor issue with limited user impact. A single feature is partially broken. Response within 1 hour.
- **SEV-4**: Cosmetic or low-priority issue. No user impact on core functionality. Addressed during normal business hours.

### Incident Response Steps

1. The on-call engineer acknowledges the alert within the defined response time for the severity level.
2. Assess the scope of the incident by checking dashboards, logs, and recent deployments.
3. If the incident correlates with a recent deployment, initiate a rollback immediately.
4. Open an incident channel in Slack and page additional responders if the issue is SEV-1 or SEV-2.
5. Communicate the current status to stakeholders through the status page.
6. Identify the root cause using traces, logs, and metrics.
7. Apply a fix or mitigation and verify the service has recovered.
8. Monitor for recurrence for at least 30 minutes after the fix.
9. Write a post-incident review within 48 hours documenting the timeline, root cause, impact, and corrective actions.
10. Track all corrective actions as tickets and assign owners with due dates.

### Communication

During an active incident, communication follows a structured cadence.

- SEV-1: Status page updated every 15 minutes. Stakeholder email at start and resolution.
- SEV-2: Status page updated every 30 minutes.
- SEV-3 and SEV-4: No external communication required.
- All incidents are summarized in the weekly operations report.

---

## Maintenance Windows

Regular maintenance is performed during low-traffic windows to minimize user impact.

- **Database maintenance**: Sundays 02:00–04:00 UTC. Includes vacuum, reindex, and minor version upgrades.
- **Infrastructure patching**: First Tuesday of each month, 03:00–05:00 UTC. OS and runtime updates across all nodes.
- **Certificate rotation**: Automated, no maintenance window required.
- **Dependency updates**: Reviewed weekly, deployed through the normal pipeline.

### Pre-Maintenance Checklist

- Notify stakeholders at least 48 hours in advance for planned downtime.
- Verify backups completed successfully within the last 24 hours.
- Confirm rollback procedure is tested and documented.
- Ensure on-call coverage is scheduled during the maintenance window.
- Stage the maintenance changes in the staging environment first.
- Prepare a communication template for the status page.

### Post-Maintenance Verification

- Run the full smoke test suite against the production endpoint.
- Check all monitoring dashboards for anomalies.
- Verify database replication lag is within acceptable limits.
- Confirm all background worker queues are processing normally.
- Send an all-clear notification to stakeholders.
