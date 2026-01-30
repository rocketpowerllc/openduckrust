# OpenDuckRust — Product Architecture

## Overview

**OpenDuckRust** (`openduckrust`) is a multi-tenant SaaS platform built on AWS
with a Rust backend, React frontend, React Native mobile app, and full CI/CD automation via CDK.

- **Proper Name**: OpenDuckRust
- **Slug / Profile**: openduckrust
- **Domain**: openduckrust.com
- **Region**: us-east-1
- **Org**: rocketpowerllc
- **Specifications**: See `specifications.md` in the project root

---

## Directory Structure

```
.
├── backend/            # Rust API (utoipa, Cedar RBAC, Lambda + ECS)
├── cdk/                # AWS CDK (TypeScript) — multi-account infrastructure
├── web-app/            # React (Vite) + auto-generated SDK
├── mobile-app/         # React Native (Expo)
├── marketing-site/     # Static HTML/CSS landing page
├── cli/                # Rust CLI tool (clap)
├── mcp/                # Model Context Protocol server
├── desktop/            # Tauri desktop wrapper
├── scripts/            # Deploy, debug, and orchestration scripts
├── specifications.md   # Product specifications (if provided)
├── claude.md           # Session config (git-ignored, secrets refs)
└── product.architecture.md  # This file
```

---

## Infrastructure (AWS CDK)

### Networking — `NetworkStack`
- VPC in **us-east-1** with public and private subnets
- NAT Gateway for private subnet egress
- VPC Endpoints for DynamoDB, S3, and Bedrock

### Authentication — `AuthStack`
- **Cognito User Pool**: email/password + Social/SSO federation
- **Cognito Identity Pool**: federated identities for AWS resource access
- Custom attributes: `tenant_id`, `org_id`, `plan`

### Security & RBAC
- **Amazon Verified Permissions** with **Cedar** policy language
- Multi-tenant filtering: every request carries `tenant_id` from JWT
- Global middleware enforces tenant isolation on all DynamoDB queries
- **KMS** encryption for DynamoDB tables and S3 buckets

### Compute — `ComputeStack`
- **Lambda** (Rust, x86_64): API handlers, versioned with aliases (keep last 5)
- **ECS Fargate**: background jobs, long-running tasks
- **AWS Batch**: heavy compute workloads, shared Docker image with ECS

### Storage — `StorageStack`
- **DynamoDB Global Tables**: primary data store, single-table design


- **S3 Buckets** (all private, KMS-encrypted):
  - `openduckrust-web-app` — React SPA assets
  - `openduckrust-marketing` — static marketing site
  - `openduckrust-assets` — user uploads, media
  - `openduckrust-vectors` — Bedrock Knowledge Base source docs

### Frontend Hosting — `FrontendStack`
- **CloudFront** distributions for web-app, marketing, and API
- Custom error responses: 404/403 → `/index.html` (SPA refresh safety)
- Origin Access Identity (OAI) for private S3 buckets
- Route 53 alias records for `openduckrust.com`

### AI — `AiStack`
- **Bedrock Knowledge Base**: S3 vector source, RAG pipeline
- **Bedrock Agents**: autonomous task execution
- Model access configured per-environment

---

## Backend Architecture (Rust)

### Framework
- **utoipa** for OpenAPI 3.0 spec generation and Swagger UI
- Runs on Actix Web locally; compiles to Lambda with `lambda_http`

### Dependency Injection
- Trait-based DI: `StorageProvider`, `AiProvider`
- Implementations: `DynamoDbProvider`, `BedrockProvider`
- Swappable for testing (mock providers)

### Multi-Tenancy
- Global middleware extracts `tenant_id` from JWT
- All DynamoDB queries automatically scoped by `tenant_id`
- Cedar policies evaluated on every request via Verified Permissions

### Observability
- Structured JSON logging via `tracing` + `tracing-subscriber`
- Log levels: INFO, WARNING, DEBUG, ERROR
- CloudWatch Log Groups: `/aws/lambda/openduckrust-api`
- Request ID propagation for end-to-end tracing

### API Spec Export
- `cargo run --bin export-swagger` generates `openapi.json`
- Web app auto-generates TypeScript SDK from this spec

---

## Frontend Architecture

### Web App (React + Vite)
- TypeScript, React 19, React Router 7
- SDK auto-generated from OpenAPI spec (`openapi-generator-cli`)
- Cognito auth integration (`amazon-cognito-identity-js`)

### Mobile App (React Native + Expo)
- Shared business logic with web where possible
- React Navigation for routing
- Expo for builds and OTA updates

### Marketing Site
- Pure static HTML/CSS
- Hosted on S3 + CloudFront
- No JavaScript framework dependency

### Desktop (Tauri)
- Wraps the React web app in a native window
- Lightweight alternative to Electron

---

## CLI Tool (Rust + Clap)

- Binary name: `openduckrust`
- Subcommands: `health`, `login`, (extend as needed)
- Communicates with the API via HTTP

---

## MCP Server

- Model Context Protocol server for Claude integration
- Exposes product data, user context, and AI tools
- Enables Claude to interact with openduckrust resources directly

---

## Deployment

### Scripts
| Script | Purpose |
|--------|---------|
| `deploy-backend.sh` | Cargo build, export swagger, prune Lambda versions, invalidate CF |
| `deploy-web.sh` | Vite build, S3 sync, CF invalidation |
| `deploy-marketing.sh` | S3 sync static site, CF invalidation |
| `get-logs.sh` | Fetch Lambda logs by request ID |
| `create-github-repo.sh` | Create private repo in rocketpowerllc |

### Environments
| Env | AWS Profile | Purpose |
|-----|-------------|---------|
| dev | `openduckrust-dev` | Development |
| staging | `openduckrust-staging` | Pre-production |
| prod | `openduckrust-prod` | Production |

### CloudFront Behaviors
- `/api/*` → Lambda Function URL / API Gateway
- `/*` → S3 (web-app bucket)
- Custom error: 403/404 → `/index.html` with 200 status

---

## Conventions & Standards

1. **Tenant isolation**: Every DB query filters by `tenant_id` — enforced in middleware, not optional
2. **Cedar RBAC**: Role/permission checks use Verified Permissions — never hard-code
3. **Lambda versioning**: Deploy creates a new version; prune keeps only the last 5
4. **Encryption**: All S3 buckets and DDB tables use KMS
5. **Logging**: Structured JSON, always include `request_id` and `tenant_id`
6. **SDK generation**: Any API change must re-export swagger and regenerate the SDK
