use async_graphql::*;

use crate::models::{ConversionRequest, ConversionResponse, DataObject, DataObjectGraphQL, Metadata};
use uuid::Uuid;

//use crate::common_utils::{RoleGuard, is_admin, UserRole};

#[derive(Default)]
pub struct DataObjectQuery;

#[Object]
impl DataObjectQuery {
    // DataObjects
    /// Returns count of DataObjects in the system
    pub async fn data_object_count(&self, _context: &Context<'_>) -> Result<i64> {
        DataObject::get_count()
    }

    /// Returns a data_object by its Uuid
    pub async fn data_object_by_id(&self, _context: &Context<'_>, id: Uuid) -> Result<DataObjectGraphQL> {
        let obj = DataObject::get_by_id(&id)?;
        Ok(obj.into())
    }

    /// Accepts a String "name" and returns a vector of data_objects that
    /// match in EN or FR against it
    pub async fn data_objects_by_title(
        &self,
        _context: &Context<'_>,
        title: String,
    ) -> Result<Vec<DataObjectGraphQL>> {
        let objects = DataObject::get_by_title(&title)?;
        Ok(objects.into_iter().map(|obj| obj.into()).collect())
    }

    /// Return a DataObjectCount by a specific DataObjectDomain (SCIENTIFIC, etc.)
    pub async fn data_object_by_metadata_domain(
        &self,
        _context: &Context<'_>,
        domain: String,
    ) -> Result<Vec<DataObjectGraphQL>> {
        let data_object_ids = Metadata::get_data_object_ids_by_domain(domain)?;
        let objects = DataObject::get_by_ids(data_object_ids)?;
        Ok(objects.into_iter().map(|obj| obj.into()).collect())
    }

    /// Returns all conversion requests associated with a specific data object
    ///
    /// A data object can have multiple conversion requests if it has been
    /// converted for different target nations or re-converted over time.
    pub async fn conversion_requests_by_data_object(
        &self,
        _context: &Context<'_>,
        data_object_id: Uuid,
    ) -> Result<ConversionRequest> {
        ConversionRequest::get_by_data_object_id(&data_object_id)
    }

    /// Returns all conversion responses associated with a specific data object (subject data)
    ///
    /// These are the classification conversion results for this data object.
    /// Each response contains the NATO equivalent and target nation classifications.
    pub async fn conversion_responses_by_data_object(
        &self,
        _context: &Context<'_>,
        data_object_id: Uuid,
    ) -> Result<ConversionResponse> {
        ConversionResponse::get_by_data_object_id(&data_object_id)
    }

    // DataObjects

    /// Returns vector of all data_objects
    pub async fn data_objects(&self, _context: &Context<'_>) -> Result<Vec<DataObjectGraphQL>> {
        let objects = DataObject::get_all()?;
        Ok(objects.into_iter().map(|obj| obj.into()).collect())
    }
}
