use actix_web::{web, HttpResponse, HttpRequest, Result};
use async_graphql::http::{GraphiQLSource};
use async_graphql::Schema;

use async_graphql_actix_web::{GraphQLSubscription,
    GraphQLRequest, GraphQLResponse};

use crate::models;
use crate::graphql::{AppSchema, ClientIP, UserAgent, QueryText, QueryVariables};


pub async fn playground_handler() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(GraphiQLSource::build().endpoint("/graphql").finish())
}

pub async fn graphql(
    schema: web::Data<AppSchema>,
    http_request: HttpRequest,
    req: GraphQLRequest,
) -> GraphQLResponse {

    let mut query = req.into_inner();

    // Extract query text and variables for audit logging BEFORE execution
    // We need to clone these because the query will be consumed
    let query_text = query.query.clone();
    let variables = query.variables.clone();

    let maybe_role_id = models::get_claim(http_request.clone());

    // insert claim data into query or error for response
    match maybe_role_id {
        Ok((role, uuid, exp_time)) => {
            query = query.data(role);
            query = query.data(uuid);
            query = query.data(exp_time)
        },
        Err(e) => {
            query = query.data(e);
        }
    };

    // Pass query text and variables to audit extension
    query = query.data(QueryText(query_text));

    // Convert variables to JSON value for logging
    let vars_json = serde_json::to_value(&variables).unwrap_or(serde_json::json!({}));
    query = query.data(QueryVariables(vars_json));

    // Extract HTTP metadata for audit logging using wrapper types
    // Client IP address
    let client_ip = http_request
        .connection_info()
        .realip_remote_addr()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    query = query.data(ClientIP(client_ip));

    // User Agent
    if let Some(user_agent) = http_request.headers().get("user-agent") {
        if let Ok(ua_str) = user_agent.to_str() {
            query = query.data(UserAgent(ua_str.to_string()));
        }
    }

    schema.execute(query).await.into()
}

pub async fn graphql_ws(
    schema: web::Data<AppSchema>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse> {
    GraphQLSubscription::new(Schema::clone(&*schema)).start(&req, payload)
}
