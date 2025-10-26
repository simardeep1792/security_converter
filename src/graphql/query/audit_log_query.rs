use async_graphql::*;
use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::models::GraphQLAuditLog;
use crate::common_utils::{RoleGuard, is_admin, UserRole};

/// Lightweight GraphQL output type for audit logs
/// This exposes the audit log data to GraphQL queries with optimized field handling
#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex)]
pub struct AuditLogGraphQL {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub user_role: Option<String>,
    pub user_access_level: Option<String>,
    pub authority_id: Option<Uuid>,
    pub nation_code: Option<String>,
    pub operation_type: String,
    pub operation_name: Option<String>,

    // Large fields - only serialize when requested
    #[graphql(skip)]
    pub query_text_raw: String,

    #[graphql(skip)]
    pub variables_json_raw: Option<serde_json::Value>,

    #[graphql(skip)]
    pub errors_json_raw: Option<serde_json::Value>,

    pub request_id: Option<Uuid>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub execution_time_ms: Option<i32>,
    pub response_status: String,
    pub error_message: Option<String>,

    #[graphql(skip)]
    pub executed_at_raw: NaiveDateTime,

    pub session_id: Option<String>,
}

#[ComplexObject]
impl AuditLogGraphQL {
    /// Full GraphQL query text (may be large - use with caution)
    async fn query_text(&self) -> String {
        self.query_text_raw.clone()
    }

    /// Query variables as JSON string
    /// May contain sensitive content - username, password, etc. 
    async fn variables_json(&self) -> Option<String> {
        self.variables_json_raw.as_ref().map(|v| v.to_string())
    }

    /// Error details as JSON string
    async fn errors_json(&self) -> Option<String> {
        self.errors_json_raw.as_ref().map(|e| e.to_string())
    }

    /// Formatted execution timestamp
    async fn executed_at(&self) -> String {
        self.executed_at_raw.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
    }

    /// ISO 8601 timestamp
    async fn executed_at_iso(&self) -> String {
        self.executed_at_raw.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
    }

    /// Unix timestamp (seconds since epoch)
    async fn executed_at_unix(&self) -> i64 {
        self.executed_at_raw.and_utc().timestamp()
    }

    /// Truncated query text (first 200 chars) for list views
    async fn query_preview(&self) -> String {
        if self.query_text_raw.len() > 200 {
            format!("{}...", &self.query_text_raw[..200])
        } else {
            self.query_text_raw.clone()
        }
    }
}

impl From<GraphQLAuditLog> for AuditLogGraphQL {
    fn from(log: GraphQLAuditLog) -> Self {
        Self {
            id: log.id,
            user_id: log.user_id,
            user_role: log.user_role,
            user_access_level: log.user_access_level,
            authority_id: log.authority_id,
            nation_code: log.nation_code,
            operation_type: log.operation_type.to_string(),
            operation_name: log.operation_name,
            query_text_raw: log.query_text,
            variables_json_raw: log.variables_json,
            errors_json_raw: log.errors_json,
            request_id: log.request_id,
            client_ip: log.client_ip,
            user_agent: log.user_agent,
            execution_time_ms: log.execution_time_ms,
            response_status: log.response_status,
            error_message: log.error_message,
            executed_at_raw: log.executed_at,
            session_id: log.session_id,
        }
    }
}

/// Input type for filtering audit logs
#[derive(InputObject)]
pub struct AuditLogFilter {
    /// Filter by user ID
    pub user_id: Option<Uuid>,

    /// Filter by authority ID
    pub authority_id: Option<Uuid>,

    /// Filter by nation code
    pub nation_code: Option<String>,

    /// Filter by operation type (query, mutation, subscription)
    pub operation_type: Option<String>,

    /// Filter by response status (success, error, partial)
    pub response_status: Option<String>,

    /// Filter by date range - start
    pub start_date: Option<String>,

    /// Filter by date range - end
    pub end_date: Option<String>,

    /// Filter by minimum execution time (ms)
    pub min_execution_time: Option<i32>,
}

#[derive(Default)]
pub struct AuditLogQuery;

#[Object]
impl AuditLogQuery {
    /// Get recent audit logs with advanced filtering (Admin only)
    ///
    /// Returns audit log entries with pagination and filtering support
    #[graphql(
        name = "auditLogs",
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    pub async fn audit_logs(
        &self,
        _context: &Context<'_>,
        #[graphql(desc = "Maximum number of logs to return", default = 50)] limit: i32,
        #[graphql(desc = "Number of records to skip for pagination", default = 0)] offset: i32,
        #[graphql(desc = "Optional filters")] filter: Option<AuditLogFilter>,
    ) -> Result<Vec<AuditLogGraphQL>> {
        // Cap limit at 1000 to prevent excessive data transfer
        let safe_limit = limit.min(1000).max(1);
        let safe_offset = offset.max(0);

        let logs = if let Some(f) = filter {
            GraphQLAuditLog::get_with_filters(safe_limit as i64, safe_offset as i64, f)?
        } else {
            GraphQLAuditLog::get_recent_paginated(safe_limit as i64, safe_offset as i64)?
        };

        Ok(logs.into_iter().map(AuditLogGraphQL::from).collect())
    }

    /// Get audit logs for a specific user (Admin only)
    ///
    /// Track all GraphQL operations performed by a specific user
    #[graphql(
        name = "auditLogsByUser",
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    pub async fn audit_logs_by_user(
        &self,
        _context: &Context<'_>,
        user_id: Uuid,
        #[graphql(desc = "Maximum number of logs to return", default = 50)] limit: i32,
        #[graphql(desc = "Number of records to skip", default = 0)] offset: i32,
    ) -> Result<Vec<AuditLogGraphQL>> {
        let safe_limit = limit.min(1000).max(1);
        let safe_offset = offset.max(0);

        let logs = GraphQLAuditLog::get_by_user_id(&user_id, safe_limit as i64, safe_offset as i64)?;
        Ok(logs.into_iter().map(AuditLogGraphQL::from).collect())
    }

    /// Get audit logs for a specific authority/organization (Admin only)
    ///
    /// Track all GraphQL operations associated with an authority
    #[graphql(
        name = "auditLogsByAuthority",
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    pub async fn audit_logs_by_authority(
        &self,
        _context: &Context<'_>,
        authority_id: Uuid,
        #[graphql(desc = "Maximum number of logs to return", default = 50)] limit: i32,
    ) -> Result<Vec<AuditLogGraphQL>> {
        let safe_limit = limit.min(1000).max(1);
        let logs = GraphQLAuditLog::get_by_authority_id(&authority_id, safe_limit as i64)?;
        Ok(logs.into_iter().map(AuditLogGraphQL::from).collect())
    }

    /// Get audit logs for a specific nation (Admin only)
    ///
    /// Track all GraphQL operations from users of a specific nation
    #[graphql(
        name = "auditLogsByNation",
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    pub async fn audit_logs_by_nation(
        &self,
        _context: &Context<'_>,
        nation_code: String,
        #[graphql(desc = "Maximum number of logs to return", default = 50)] limit: i32,
    ) -> Result<Vec<AuditLogGraphQL>> {
        let safe_limit = limit.min(1000).max(1);
        let logs = GraphQLAuditLog::get_by_nation_code(&nation_code, safe_limit as i64)?;
        Ok(logs.into_iter().map(AuditLogGraphQL::from).collect())
    }

    /// Get all failed operations for security review (Admin only)
    ///
    /// Returns audit logs where the operation resulted in an error
    #[graphql(
        name = "failedOperations",
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    pub async fn failed_operations(
        &self,
        _context: &Context<'_>,
        #[graphql(desc = "Maximum number of logs to return", default = 50)] limit: i32,
    ) -> Result<Vec<AuditLogGraphQL>> {
        let safe_limit = limit.min(1000).max(1);
        let logs = GraphQLAuditLog::get_failed_operations(safe_limit as i64)?;
        Ok(logs.into_iter().map(AuditLogGraphQL::from).collect())
    }

    /// Get audit logs by request ID for correlation (Admin only)
    ///
    /// Track all operations associated with a specific request ID
    #[graphql(
        name = "auditLogsByRequestId",
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    pub async fn audit_logs_by_request_id(
        &self,
        _context: &Context<'_>,
        request_id: Uuid,
    ) -> Result<Vec<AuditLogGraphQL>> {
        let logs = GraphQLAuditLog::get_by_request_id(&request_id)?;
        Ok(logs.into_iter().map(AuditLogGraphQL::from).collect())
    }

    /// Get audit logs for the current user (Self-service)
    ///
    /// Allows users to view their own activity log
    #[graphql(name = "myAuditLogs")]
    pub async fn my_audit_logs(
        &self,
        context: &Context<'_>,
        #[graphql(desc = "Maximum number of logs to return", default = 20)] limit: i32,
    ) -> Result<Vec<AuditLogGraphQL>> {
        // Get the current user's UUID from context
        let user_id = context.data_opt::<Uuid>()
            .ok_or_else(|| Error::new("User not authenticated"))?;

        let safe_limit = limit.min(100).max(1); // Users get lower limit
        let logs = GraphQLAuditLog::get_by_user_id(user_id, safe_limit as i64, 0)?;
        Ok(logs.into_iter().map(AuditLogGraphQL::from).collect())
    }
}
