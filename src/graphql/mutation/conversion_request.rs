use async_graphql::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{
    ConversionRequest, ConversionResponse, InsertableConversionRequest,
};
use crate::common_utils::{UserRole, is_admin, RoleGuard, is_user};

#[derive(Default)]
pub struct ConversionRequestMutation;

#[derive(Debug, Serialize, Deserialize, SimpleObject)]
pub struct ConversionRequestResponse {
    pub request: ConversionRequest,
    pub response: Option<ConversionResponse>,
    pub success: bool,
    pub message: String,
}

#[Object]
impl ConversionRequestMutation {
    /// Submit a new conversion request for security classification translation
    ///
    /// This mutation:
    /// 1. Creates a DataObject with the provided document information
    /// 2. Creates Metadata for the document
    /// 3. Creates a ConversionRequest record
    /// 4. Automatically processes the conversion (source → NATO → targets)
    /// 5. Returns both the request and response
    ///
    /// Requires authentication. The user must belong to a valid authority.
    #[graphql(
        name = "submitConversionRequest",
        guard = "RoleGuard::new(UserRole::User)",
        visible = "is_user",
    )]
    pub async fn submit_conversion_request(
        &self,
        context: &Context<'_>,
        conversion_data: InsertableConversionRequest,
    ) -> Result<ConversionRequestResponse> {
        // Get the authenticated user ID from context
        let user_id = context.data_opt::<Uuid>();

        if user_id.is_none() {
            return Err(Error::new("User not authenticated. Please provide a valid JWT token."));
        }

        // Verify the user_id in the payload matches the authenticated user
        // (or allow admins to submit on behalf of others)
        // For now, we'll trust the payload but this should be validated in production

        // Step 1: Process the payload to create the conversion request
        match ConversionRequest::process_payload(&conversion_data) {
            Ok(mut request) => {
                // Step 2: Automatically process the conversion
                match request.process_and_convert() {
                    Ok(response) => {
                        Ok(ConversionRequestResponse {
                            request,
                            response: Some(response),
                            success: true,
                            message: "Conversion request submitted and processed successfully".to_string(),
                        })
                    }
                    Err(e) => {
                        // Conversion failed but request was created
                        Ok(ConversionRequestResponse {
                            request,
                            response: None,
                            success: false,
                            message: format!("Conversion request created but processing failed: {:?}", e),
                        })
                    }
                }
            }
            Err(e) => {
                Err(Error::new(format!("Failed to create conversion request: {:?}", e)))
            }
        }
    }

    /// Submit a conversion request without automatic processing
    ///
    /// This creates the request record but does not immediately process the conversion.
    /// Useful for batch processing or manual review workflows.
    ///
    /// Requires authentication.
    #[graphql(
        name = "createConversionRequest",
        guard = "RoleGuard::new(UserRole::User)",
        visible = "is_user",
    )]
    pub async fn create_conversion_request(
        &self,
        context: &Context<'_>,
        conversion_data: InsertableConversionRequest,
    ) -> Result<ConversionRequest> {
        let user_id = context.data_opt::<Uuid>();

        if user_id.is_none() {
            return Err(Error::new("User not authenticated. Please provide a valid JWT token."));
        }

        ConversionRequest::process_payload(&conversion_data)
    }

    /// Process an existing conversion request to generate a response
    ///
    /// Takes a conversion request ID and processes it to generate classification conversions.
    /// This is useful for:
    /// - Re-processing failed conversions
    /// - Processing requests that were created without automatic conversion
    /// - Updating conversions when schemas change
    ///
    /// Requires authentication.
    #[graphql(
        name = "processConversionRequest",
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    pub async fn process_conversion_request(
        &self,
        _context: &Context<'_>,
        request_id: Uuid,
    ) -> Result<ConversionResponse> {
        let mut request = ConversionRequest::get_by_id(&request_id)?;
        request.process_and_convert()
    }

    /// Mark a conversion request as completed
    ///
    /// Updates the completed_at timestamp for a conversion request.
    /// This is typically done automatically but can be triggered manually if needed.
    ///
    /// Requires admin privileges.
    #[graphql(
        name = "markConversionRequestCompleted",
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    pub async fn mark_conversion_request_completed(
        &self,
        _context: &Context<'_>,
        request_id: Uuid,
    ) -> Result<ConversionRequest> {
        let mut request = ConversionRequest::get_by_id(&request_id)?;
        request.mark_completed()
    }

    /// Delete a conversion request and its associated data
    ///
    /// This will delete:
    /// - The conversion request record
    /// - Any associated conversion responses (via CASCADE)
    ///
    /// Note: This does NOT delete the DataObject or Metadata as they may be referenced elsewhere.
    ///
    /// Requires admin privileges.
    #[graphql(
        name = "deleteConversionRequest",
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    pub async fn delete_conversion_request(
        &self,
        _context: &Context<'_>,
        request_id: Uuid,
    ) -> Result<String> {
        let request = ConversionRequest::get_by_id(&request_id)?;
        request.delete()?;
        Ok(format!("Successfully deleted conversion request {}", request_id))
    }
}
