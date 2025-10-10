use async_graphql::*;

use crate::graphql::{ClassificationSchemaQuery, ConversionRequestQuery, DataObjectQuery, NationQuery, query::UserQuery};

#[derive(Default, MergedObject)]
pub struct Query(
    UserQuery,
    DataObjectQuery,
    NationQuery,
    ClassificationSchemaQuery,
    ConversionRequestQuery,
);
