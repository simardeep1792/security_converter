use async_graphql::extensions::{Extension, ExtensionContext, ExtensionFactory, NextExecute};
use async_graphql::Response;
use std::sync::Arc;
use uuid::Uuid;
use serde_json::json;

use crate::models::{InsertableGraphQLAuditLog, GraphQLAuditLog, GraphqlOperationType};
use crate::common_utils::UserRole;

// Wrapper types to distinguish different string data in context
#[derive(Clone, Debug)]
pub struct ClientIP(pub String);

#[derive(Clone, Debug)]
pub struct UserAgent(pub String);

#[derive(Clone, Debug)]
pub struct QueryText(pub String);

#[derive(Clone, Debug)]
pub struct QueryVariables(pub serde_json::Value);

/// Extension that logs all GraphQL operations to the database for security auditing and analytics
pub struct AuditLogExtension;

impl ExtensionFactory for AuditLogExtension {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(AuditLogExtensionImpl {
            start_time: std::sync::Mutex::new(None),
        })
    }
}

struct AuditLogExtensionImpl {
    start_time: std::sync::Mutex<Option<std::time::Instant>>,
}

#[async_trait::async_trait]
impl Extension for AuditLogExtensionImpl {
    /// Called when GraphQL execution completes
    async fn execute(
        &self,
        ctx: &ExtensionContext<'_>,
        operation_name: Option<&str>,
        next: NextExecute<'_>,
    ) -> Response {
        // Record start time
        *self.start_time.lock().unwrap() = Some(std::time::Instant::now());

        let response = next.run(ctx, operation_name).await;

        // Calculate execution time
        let execution_time_ms = self
            .start_time
            .lock()
            .unwrap()
            .map(|start| start.elapsed().as_millis() as i32);

        // Extract user context from the GraphQL context
        let user_id: Option<Uuid> = ctx.data_opt::<Uuid>().copied();
        // Convert UserRole enum to string for logging
        let user_role: Option<String> = ctx.data_opt::<UserRole>().map(|r| match r {
            UserRole::User => "User",
            UserRole::Operator => "Operator",
            UserRole::Analyst => "Analyst",
            UserRole::Admin => "Admin",
        }.to_string());

        // Extract error info if present
        let (response_status, error_message, errors_json) = if response.is_err() {
            let errors: Vec<_> = response.errors.iter().map(|e| {
                json!({
                    "message": e.message.clone(),
                    "path": e.path.clone(),
                    "locations": e.locations.clone(),
                })
            }).collect();

            let first_error = response.errors.first().map(|e| e.message.clone());

            (
                "error".to_string(),
                first_error,
                Some(serde_json::Value::Array(errors)),
            )
        } else if !response.errors.is_empty() {
            // Partial errors
            let errors: Vec<_> = response.errors.iter().map(|e| {
                json!({
                    "message": e.message.clone(),
                    "path": e.path.clone(),
                })
            }).collect();

            (
                "partial".to_string(),
                None,
                Some(serde_json::Value::Array(errors)),
            )
        } else {
            ("success".to_string(), None, None)
        };

        // Extract query text and variables from context
        let query_text = ctx.data_opt::<QueryText>()
            .map(|q| q.0.clone())
            .unwrap_or_else(|| operation_name.unwrap_or("unknown").to_string());

        let variables_json = ctx.data_opt::<QueryVariables>()
            .map(|v| v.0.clone());

        // Determine operation type based on the query text
        let operation_type = if query_text.trim().starts_with("mutation") {
            GraphqlOperationType::Mutation
        } else if query_text.trim().starts_with("subscription") {
            GraphqlOperationType::Subscription
        } else {
            GraphqlOperationType::Query
        };

        // Extract HTTP metadata passed from the handler using wrapper types
        let client_ip = ctx.data_opt::<ClientIP>().map(|ip| ip.0.clone());
        let user_agent = ctx.data_opt::<UserAgent>().map(|ua| ua.0.clone());

        // Build audit log entry
        let audit_log = InsertableGraphQLAuditLog {
            user_id,
            user_role,
            user_access_level: None,
            authority_id: None,
            nation_code: None,
            operation_type,
            operation_name: operation_name.map(|s| s.to_string()),
            query_text,
            variables_json,
            request_id: Some(Uuid::new_v4()),
            client_ip,
            user_agent,
            execution_time_ms,
            response_status,
            error_message,
            errors_json,
            accessed_data_objects: None,
            accessed_classifications: None,
            session_id: None,
            request_headers: None,
        };

        // Asynchronously log to database (fire and forget to avoid blocking response)
        actix_rt::spawn(async move {
            if let Err(e) = GraphQLAuditLog::create(audit_log) {
                eprintln!("Failed to create audit log: {:?}", e);
            }
        });

        response
    }
}
