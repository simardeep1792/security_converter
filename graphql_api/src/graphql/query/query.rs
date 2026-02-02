use async_graphql::*;

use crate::graphql::{AuthorityQuery, ClassificationSchemaQuery, ConversionRequestQuery, DataObjectQuery, NationQuery, query::UserQuery};

#[derive(Default, MergedObject)]
pub struct Query(
    UserQuery,
    DataObjectQuery,
    NationQuery,
    AuthorityQuery,
    ClassificationSchemaQuery,
    ConversionRequestQuery,
);
