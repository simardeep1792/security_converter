use chrono::NaiveDateTime;
use diesel::{self, ExpressionMethods, Insertable, QueryDsl, Queryable, RunQueryDsl};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;
use diesel_derive_enum::DbEnum;
use async_graphql::Result;

use crate::database::connection;
use crate::schema::graphql_audit_logs;
use crate::graphql::query::{AuditLogFilter};

#[derive(Debug, Clone, PartialEq, DbEnum, Serialize, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::GraphqlOperationType"]
pub enum GraphqlOperationType {
    Query,
    Mutation,
    Subscription,
}

impl std::fmt::Display for GraphqlOperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphqlOperationType::Query => write!(f, "query"),
            GraphqlOperationType::Mutation => write!(f, "mutation"),
            GraphqlOperationType::Subscription => write!(f, "subscription"),
        }
    }
}

impl GraphqlOperationType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "query" => Some(GraphqlOperationType::Query),
            "mutation" => Some(GraphqlOperationType::Mutation),
            "subscription" => Some(GraphqlOperationType::Subscription),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Queryable)]
pub struct GraphQLAuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub user_role: Option<String>,
    pub user_access_level: Option<String>,
    pub authority_id: Option<Uuid>,
    pub nation_code: Option<String>,
    pub operation_type: GraphqlOperationType,
    pub operation_name: Option<String>,
    pub query_text: String,
    pub variables_json: Option<JsonValue>,
    pub request_id: Option<Uuid>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub execution_time_ms: Option<i32>,
    pub response_status: String,
    pub error_message: Option<String>,
    pub errors_json: Option<JsonValue>,
    pub accessed_data_objects: Option<Vec<Option<Uuid>>>,
    pub accessed_classifications: Option<Vec<Option<String>>>,
    pub executed_at: NaiveDateTime,
    pub session_id: Option<String>,
    pub request_headers: Option<JsonValue>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable)]
#[diesel(table_name = graphql_audit_logs)]
pub struct InsertableGraphQLAuditLog {
    pub user_id: Option<Uuid>,
    pub user_role: Option<String>,
    pub user_access_level: Option<String>,
    pub authority_id: Option<Uuid>,
    pub nation_code: Option<String>,
    pub operation_type: GraphqlOperationType,
    pub operation_name: Option<String>,
    pub query_text: String,
    pub variables_json: Option<JsonValue>,
    pub request_id: Option<Uuid>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub execution_time_ms: Option<i32>,
    pub response_status: String,
    pub error_message: Option<String>,
    pub errors_json: Option<JsonValue>,
    pub accessed_data_objects: Option<Vec<Option<Uuid>>>,
    pub accessed_classifications: Option<Vec<Option<String>>>,
    pub session_id: Option<String>,
    pub request_headers: Option<JsonValue>,
}

impl GraphQLAuditLog {
    /// Create a new audit log entry (optimized for fire-and-forget async logging)
    pub fn create(log: InsertableGraphQLAuditLog) -> Result<Self> {
        let mut conn = connection()?;
        let result = diesel::insert_into(graphql_audit_logs::table)
            .values(&log)
            .get_result(&mut conn)?;

        Ok(result)
    }

    /// Get audit logs for a specific user with pagination
    pub fn get_by_user_id(user_id: &Uuid, limit: i64, offset: i64) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let logs = graphql_audit_logs::table
            .filter(graphql_audit_logs::user_id.eq(user_id))
            .order(graphql_audit_logs::executed_at.desc())
            .limit(limit)
            .offset(offset)
            .load::<GraphQLAuditLog>(&mut conn)?;

        Ok(logs)
    }

    /// Get audit logs by authority
    pub fn get_by_authority_id(authority_id: &Uuid, limit: i64) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let logs = graphql_audit_logs::table
            .filter(graphql_audit_logs::authority_id.eq(authority_id))
            .order(graphql_audit_logs::executed_at.desc())
            .limit(limit)
            .load::<GraphQLAuditLog>(&mut conn)?;

        Ok(logs)
    }

    /// Get audit logs by nation code
    pub fn get_by_nation_code(nation_code: &str, limit: i64) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let logs = graphql_audit_logs::table
            .filter(graphql_audit_logs::nation_code.eq(nation_code))
            .order(graphql_audit_logs::executed_at.desc())
            .limit(limit)
            .load::<GraphQLAuditLog>(&mut conn)?;

        Ok(logs)
    }

    /// Get all failed operations for security review
    pub fn get_failed_operations(limit: i64) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let logs = graphql_audit_logs::table
            .filter(graphql_audit_logs::response_status.eq("error"))
            .order(graphql_audit_logs::executed_at.desc())
            .limit(limit)
            .load::<GraphQLAuditLog>(&mut conn)?;

        Ok(logs)
    }

    /// Get audit logs by request ID (for tracking correlated requests)
    pub fn get_by_request_id(request_id: &Uuid) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let logs = graphql_audit_logs::table
            .filter(graphql_audit_logs::request_id.eq(request_id))
            .order(graphql_audit_logs::executed_at.asc())
            .load::<GraphQLAuditLog>(&mut conn)?;

        Ok(logs)
    }

    /// Get recent audit logs with pagination (NEW)
    pub fn get_recent_paginated(limit: i64, offset: i64) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let logs = graphql_audit_logs::table
            .order(graphql_audit_logs::executed_at.desc())
            .limit(limit)
            .offset(offset)
            .load::<GraphQLAuditLog>(&mut conn)?;

        Ok(logs)
    }

    /// Get recent audit logs with optional filters (LEGACY - kept for backwards compatibility)
    pub fn get_recent(
        limit: i64,
        operation_type: Option<GraphqlOperationType>,
    ) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let mut query = graphql_audit_logs::table.into_boxed();

        if let Some(op_type) = operation_type {
            query = query.filter(graphql_audit_logs::operation_type.eq(op_type));
        }

        let logs = query
            .order(graphql_audit_logs::executed_at.desc())
            .limit(limit)
            .load::<GraphQLAuditLog>(&mut conn)?;

        Ok(logs)
    }

    /// Get audit logs with advanced filtering (NEW)
    pub fn get_with_filters(limit: i64, offset: i64, filter: AuditLogFilter) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let mut query = graphql_audit_logs::table.into_boxed();

        // Apply filters
        if let Some(user_id) = filter.user_id {
            query = query.filter(graphql_audit_logs::user_id.eq(user_id));
        }

        if let Some(authority_id) = filter.authority_id {
            query = query.filter(graphql_audit_logs::authority_id.eq(authority_id));
        }

        if let Some(nation_code) = filter.nation_code {
            query = query.filter(graphql_audit_logs::nation_code.eq(nation_code));
        }

        if let Some(op_type_str) = filter.operation_type {
            if let Some(op_type) = GraphqlOperationType::from_str(&op_type_str) {
                query = query.filter(graphql_audit_logs::operation_type.eq(op_type));
            }
        }

        if let Some(status) = filter.response_status {
            query = query.filter(graphql_audit_logs::response_status.eq(status));
        }

        if let Some(start_date) = filter.start_date {
            if let Ok(date) = NaiveDateTime::parse_from_str(&format!("{} 00:00:00", start_date), "%Y-%m-%d %H:%M:%S") {
                query = query.filter(graphql_audit_logs::executed_at.ge(date));
            }
        }

        if let Some(end_date) = filter.end_date {
            if let Ok(date) = NaiveDateTime::parse_from_str(&format!("{} 23:59:59", end_date), "%Y-%m-%d %H:%M:%S") {
                query = query.filter(graphql_audit_logs::executed_at.le(date));
            }
        }

        if let Some(min_time) = filter.min_execution_time {
            query = query.filter(graphql_audit_logs::execution_time_ms.ge(min_time));
        }

        let logs = query
            .order(graphql_audit_logs::executed_at.desc())
            .limit(limit)
            .offset(offset)
            .load::<GraphQLAuditLog>(&mut conn)?;

        Ok(logs)
    }
}
