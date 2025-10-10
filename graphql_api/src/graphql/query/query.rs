use async_graphql::*;

use crate::graphql::{ClassificationSchemaQuery, ConversionRequestQuery, ConversionResponseQuery, DataObjectQuery, MetadataQuery, NationQuery, query::UserQuery};

#[derive(Default, MergedObject)]
pub struct Query(
    UserQuery,
    DataObjectQuery,
    MetadataQuery,
    NationQuery,
    ClassificationSchemaQuery,
    ConversionRequestQuery,
    ConversionResponseQuery,
);
