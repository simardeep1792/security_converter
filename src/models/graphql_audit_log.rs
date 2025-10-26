use chrono::NaiveDateTime;
use diesel::{self, ExpressionMethods, Insertable, QueryDsl, Queryable, RunQueryDsl};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;
use diesel_derive_enum::DbEnum;
use async_graphql::Result;

use crate::database::connection;
use crate::schema::graphql_audit_logs;

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
    /// Create a new audit log entry
    pub fn create(log: InsertableGraphQLAuditLog) -> Result<Self> {
        let mut conn = connection()?;
        let result = diesel::insert_into(graphql_audit_logs::table)
            .values(&log)
            .get_result(&mut conn)?;

        Ok(result)
    }

    /// Get audit logs for a specific user
    pub fn get_by_user_id(user_id: &Uuid, limit: i64) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let logs = graphql_audit_logs::table
            .filter(graphql_audit_logs::user_id.eq(user_id))
            .order(graphql_audit_logs::executed_at.desc())
            .limit(limit)
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

    /// Get recent audit logs with optional filters
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
}
