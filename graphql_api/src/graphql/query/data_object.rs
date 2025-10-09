use async_graphql::*;

use crate::models::{DataObject, Metadata};
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
    pub async fn data_object_by_id(&self, _context: &Context<'_>, id: Uuid) -> Result<DataObject> {
        DataObject::get_by_id(&id)
    }

    /// Accepts a String "name" and returns a vector of data_objects that
    /// match in EN or FR against it
    pub async fn data_objects_by_title(
        &self,
        _context: &Context<'_>,
        title: String,
    ) -> Result<Vec<DataObject>> {
        DataObject::get_by_title(&title)
    }

    /// Return a DataObjectCount by a specific DataObjectDomain (SCIENTIFIC, etc.)
    pub async fn data_object_counts_by_metadata_domain(
        &self,
        _context: &Context<'_>,
        domain: String,
    ) -> Result<Vec<DataObject>> {
        let data_object_ids = Metadata::get_data_object_ids_by_domain(domain)?;

        DataObject::get_by_ids(data_object_ids)
    }

    // DataObjects

    /// Returns vector of all data_objects
    pub async fn data_objects(&self, _context: &Context<'_>) -> Result<Vec<DataObject>> {
        DataObject::get_all()
    }
}
