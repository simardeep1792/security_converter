use async_graphql::*;

use crate::graphql::{ClassificationSchemaQuery, DataObjectQuery, NationQuery, query::UserQuery};

#[derive(Default, MergedObject)]
pub struct Query(
    UserQuery,
    DataObjectQuery,
    NationQuery,
    ClassificationSchemaQuery,
);
