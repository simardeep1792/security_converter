use async_graphql::*;
use uuid::Uuid;

use crate::models::{GraphQLAuditLog, User, Authority, Nation};
use crate::common_utils::{RoleGuard, is_admin, UserRole};

/// GraphQL output type for audit logs
/// This exposes the audit log data to GraphQL queries
#[derive(Debug, Clone, SimpleObject)]
pub struct AuditLogGraphQL {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub user_role: Option<String>,
    pub user_access_level: Option<String>,
    pub authority_id: Option<Uuid>,
    pub nation_code: Option<String>,
    pub operation_type: String,
    pub operation_name: Option<String>,
    pub query_text: String,
    pub variables_json: Option<String>,
    pub request_id: Option<Uuid>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub execution_time_ms: Option<i32>,
    pub response_status: String,
    pub error_message: Option<String>,
    pub errors_json: Option<String>,
    pub executed_at: String,
    pub session_id: Option<String>,
}

#[ComplexObject]
impl AuditLogGraphQL {
    pub async fn user(&self) -> Result<User> {
        User::get_by_id(&self.user_id.expect("No user found."))
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
            query_text: log.query_text,
            variables_json: log.variables_json.map(|v| v.to_string()),
            request_id: log.request_id,
            client_ip: log.client_ip,
            user_agent: log.user_agent,
            execution_time_ms: log.execution_time_ms,
            response_status: log.response_status,
            error_message: log.error_message,
            errors_json: log.errors_json.map(|e| e.to_string()),
            executed_at: log.executed_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            session_id: log.session_id,
        }
    }
}

#[derive(Default)]
pub struct AuditLogQuery;

#[Object]
impl AuditLogQuery {
    /// Get recent audit logs (Admin only)
    ///
    /// Returns the most recent audit log entries with optional filtering
    #[graphql(
        name = "auditLogs",
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    pub async fn audit_logs(
        &self,
        _context: &Context<'_>,
        #[graphql(desc = "Maximum number of logs to return", default = 50)] limit: i32,
    ) -> Result<Vec<AuditLogGraphQL>> {
        let logs = GraphQLAuditLog::get_recent(limit as i64, None)?;
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
    ) -> Result<Vec<AuditLogGraphQL>> {
        let logs = GraphQLAuditLog::get_by_user_id(&user_id, limit as i64)?;
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
        let logs = GraphQLAuditLog::get_by_authority_id(&authority_id, limit as i64)?;
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
        let logs = GraphQLAuditLog::get_by_nation_code(&nation_code, limit as i64)?;
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
        let logs = GraphQLAuditLog::get_failed_operations(limit as i64)?;
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

        let logs = GraphQLAuditLog::get_by_user_id(user_id, limit as i64)?;
        Ok(logs.into_iter().map(AuditLogGraphQL::from).collect())
    }
}
