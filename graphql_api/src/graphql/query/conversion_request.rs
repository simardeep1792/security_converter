use async_graphql::*;

use crate::models::ConversionRequest;
use uuid::Uuid;

//use crate::common_utils::{RoleGuard, is_admin, UserRole};

#[derive(Default)]
pub struct ConversionRequestQuery;

#[Object]
impl ConversionRequestQuery {
    /// Returns count of ConversionRequests in the system
    pub async fn conversion_request_count(&self, _context: &Context<'_>) -> Result<i64> {
        let requests = ConversionRequest::get_all()?;
        Ok(requests.len() as i64)
    }

    /// Returns a conversion request by its Uuid
    pub async fn conversion_request_by_id(
        &self,
        _context: &Context<'_>,
        id: Uuid,
    ) -> Result<ConversionRequest> {
        ConversionRequest::get_by_id(&id)
    }

    /// Returns conversion requests by creator ID
    pub async fn conversion_requests_by_creator_id(
        &self,
        _context: &Context<'_>,
        creator_id: Uuid,
    ) -> Result<Vec<ConversionRequest>> {
        ConversionRequest::get_by_creator_id(&creator_id)
    }

    /// Returns conversion requests by authority ID
    pub async fn conversion_requests_by_authority_id(
        &self,
        _context: &Context<'_>,
        authority_id: Uuid,
    ) -> Result<Vec<ConversionRequest>> {
        ConversionRequest::get_by_authority_id(&authority_id)
    }

    /// Returns conversion requests by data object ID
    pub async fn conversion_requests_by_data_object_id(
        &self,
        _context: &Context<'_>,
        data_object_id: Uuid,
    ) -> Result<Vec<ConversionRequest>> {
        ConversionRequest::get_by_data_object_id(&data_object_id)
    }

    /// Returns conversion requests by source nation code
    pub async fn conversion_requests_by_source_nation_code(
        &self,
        _context: &Context<'_>,
        nation_code: String,
    ) -> Result<Vec<ConversionRequest>> {
        ConversionRequest::get_by_source_nation_code(&nation_code)
    }

    /// Returns all pending (not completed) conversion requests
    pub async fn conversion_requests_pending(
        &self,
        _context: &Context<'_>,
    ) -> Result<Vec<ConversionRequest>> {
        ConversionRequest::get_pending()
    }

    /// Returns all completed conversion requests
    pub async fn conversion_requests_completed(
        &self,
        _context: &Context<'_>,
    ) -> Result<Vec<ConversionRequest>> {
        ConversionRequest::get_completed()
    }

    /// Returns vector of all conversion requests
    pub async fn conversion_requests(&self, _context: &Context<'_>) -> Result<Vec<ConversionRequest>> {
        ConversionRequest::get_all()
    }

    /// Returns a limited number of conversion requests
    pub async fn conversion_requests_count(
        &self,
        _context: &Context<'_>,
        count: i64,
    ) -> Result<Vec<ConversionRequest>> {
        ConversionRequest::get_count(count)
    }
}
