use async_graphql::*;
use uuid::Uuid;

use crate::models::{ConversionRequest, InsertableConversionRequest, InsertableDataObject, InsertableMetadata};
use crate::common_utils::{UserRole, is_admin, is_operator, RoleGuard};

#[derive(Default)]
pub struct ConversionRequestMutation;

/// Input type for submitting a new conversion request via GraphQL
#[derive(Debug, InputObject)]
pub struct SubmitConversionRequestInput {
    /// The authority ID requesting the conversion
    pub authority_id: Uuid,
    /// Title of the data object being classified
    pub data_object_title: String,
    /// Description of the data object
    pub data_object_description: String,
    /// Metadata domain (e.g., SCIENTIFIC, MILITARY, DIPLOMATIC)
    pub metadata_domain: String,
    /// Tags for the metadata
    pub metadata_tags: Vec<String>,
    /// Source nation code (e.g., "USA", "CAN", "GBR")
    pub source_nation_code: String,
    /// Target nation codes for conversion
    pub target_nation_codes: Vec<String>,
}

#[Object]
impl ConversionRequestMutation {
    #[graphql(
        name = "submitConversionRequest",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    /// Submit a new conversion request. Requires operator role or higher.
    /// This creates a data object, metadata, and the conversion request in one transaction.
    pub async fn submit_conversion_request(
        &self,
        context: &Context<'_>,
        input: SubmitConversionRequestInput,
    ) -> Result<ConversionRequest> {
        // Get the user ID from the context (set by auth middleware)
        let user_id = context
            .data::<uuid::Uuid>()
            .map_err(|_| Error::new("User not authenticated"))?;

        // Convert Vec<String> to Vec<Option<String>> for metadata tags
        let tags_with_options: Vec<Option<String>> = input.metadata_tags
            .into_iter()
            .map(Some)
            .collect();

        let payload = InsertableConversionRequest {
            user_id: *user_id,
            authority_id: input.authority_id,
            data_object: InsertableDataObject {
                title: input.data_object_title,
                description: input.data_object_description,
            },
            metadata: InsertableMetadata {
                domain: input.metadata_domain,
                tags: tags_with_options,
            },
            source_nation_code: input.source_nation_code,
            target_nation_codes: input.target_nation_codes,
        };

        ConversionRequest::process_payload(&payload)
    }

    #[graphql(
        name = "markConversionRequestCompleted",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    /// Mark a conversion request as completed. Requires operator role or higher.
    pub async fn mark_conversion_request_completed(
        &self,
        _context: &Context<'_>,
        id: Uuid,
    ) -> Result<ConversionRequest> {
        let mut request = ConversionRequest::get_by_id(&id)?;
        request.mark_completed()
    }

    #[graphql(
        name = "deleteConversionRequest",
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    /// Delete a conversion request. Requires admin role.
    pub async fn delete_conversion_request(
        &self,
        _context: &Context<'_>,
        id: Uuid,
    ) -> Result<bool> {
        let request = ConversionRequest::get_by_id(&id)?;
        request.delete()?;
        Ok(true)
    }
}
