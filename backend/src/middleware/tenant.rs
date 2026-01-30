//! Multi-tenancy middleware — extracts tenant_id from JWT and injects into request extensions.
//! All DynamoDB queries are filtered by tenant_id via this global middleware.

use actix_web::{dev::ServiceRequest, Error, HttpMessage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    pub tenant_id: String,
    pub user_id: String,
    pub roles: Vec<String>,
}

pub async fn extract_tenant(req: &ServiceRequest) -> Result<TenantContext, Error> {
    // TODO: Decode JWT from Authorization header
    // TODO: Extract tenant_id, user_id, roles from claims
    // TODO: Verify with Amazon Verified Permissions (Cedar)
    todo!("Implement JWT tenant extraction + Cedar RBAC check")
}
