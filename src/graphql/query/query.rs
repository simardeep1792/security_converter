use async_graphql::*;

use crate::graphql::{AuditLogQuery, ClassificationSchemaQuery, ConversionRequestQuery, ConversionResponseQuery, DataObjectQuery, MetadataQuery, NationQuery, query::UserQuery};

#[derive(Default, MergedObject)]
pub struct Query(
    UserQuery,
    AuditLogQuery,
    DataObjectQuery,
    MetadataQuery,
    NationQuery,
    ClassificationSchemaQuery,
    ConversionRequestQuery,
    ConversionResponseQuery,
);
