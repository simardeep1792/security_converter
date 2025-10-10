use async_graphql::*;

use crate::models::ConversionResponse;
use uuid::Uuid;

//use crate::common_utils::{RoleGuard, is_admin, UserRole};

#[derive(Default)]
pub struct ConversionResponseQuery;

#[Object]
impl ConversionResponseQuery {
    /// Returns count of ConversionResponses in the system
    pub async fn conversion_response_count(&self, _context: &Context<'_>) -> Result<i64> {
        let responses = ConversionResponse::get_all()?;
        Ok(responses.len() as i64)
    }

    /// Returns a conversion response by its Uuid
    pub async fn conversion_response_by_id(
        &self,
        _context: &Context<'_>,
        id: Uuid,
    ) -> Result<ConversionResponse> {
        ConversionResponse::get_by_id(&id)
    }

    /// Returns conversion responses by conversion request ID
    pub async fn conversion_response_by_request_id(
        &self,
        _context: &Context<'_>,
        request_id: Uuid,
    ) -> Result<ConversionResponse> {
        ConversionResponse::get_by_conversion_request_id(&request_id)
    }

    /// Returns conversion responses by subject data (data object) ID
    pub async fn conversion_responses_by_subject_data_id(
        &self,
        _context: &Context<'_>,
        subject_data_id: Uuid,
    ) -> Result<ConversionResponse> {
        ConversionResponse::get_by_data_object_id(&subject_data_id)
    }

    /// Returns conversion responses by NATO equivalent classification level
    pub async fn conversion_responses_by_nato_equivalent(
        &self,
        _context: &Context<'_>,
        nato_classification: String,
    ) -> Result<Vec<ConversionResponse>> {
        ConversionResponse::get_by_nato_equivalent(&nato_classification)
    }

    /// Returns all active (non-expired) conversion responses
    pub async fn conversion_responses_active(
        &self,
        _context: &Context<'_>,
    ) -> Result<Vec<ConversionResponse>> {
        ConversionResponse::get_active()
    }

    /// Returns all expired conversion responses
    pub async fn conversion_responses_expired(
        &self,
        _context: &Context<'_>,
    ) -> Result<Vec<ConversionResponse>> {
        ConversionResponse::get_expired()
    }

    /// Returns vector of all conversion responses
    pub async fn conversion_responses(&self, _context: &Context<'_>) -> Result<Vec<ConversionResponse>> {
        ConversionResponse::get_all()
    }

    /// Returns a limited number of conversion responses
    pub async fn conversion_responses_count(
        &self,
        _context: &Context<'_>,
        count: i64,
    ) -> Result<Vec<ConversionResponse>> {
        ConversionResponse::get_count(count)
    }
}
