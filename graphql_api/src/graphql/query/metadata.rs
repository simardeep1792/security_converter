use async_graphql::*;

use crate::models::{DataObject, DomainCount, Metadata};
use uuid::Uuid;

//use crate::common_utils::{RoleGuard, is_admin, UserRole};

#[derive(Default)]
pub struct MetadataQuery;

#[Object]
impl MetadataQuery {
    /// Returns count of Metadata records in the system
    pub async fn metadata_count(&self, _context: &Context<'_>) -> Result<i64> {
        let metadata = Metadata::get_all()?;
        Ok(metadata.len() as i64)
    }

    /// Returns a metadata record by its Uuid
    pub async fn metadata_by_id(&self, _context: &Context<'_>, id: Uuid) -> Result<Metadata> {
        Metadata::get_by_id(&id)
    }

    /// Returns metadata by data object ID
    ///
    /// Each data object has exactly one metadata record associated with it.
    pub async fn metadata_by_data_object_id(
        &self,
        _context: &Context<'_>,
        data_object_id: Uuid,
    ) -> Result<Metadata> {
        Metadata::get_by_data_object_id(&data_object_id)
    }

    /// Returns all metadata records for a specific domain
    ///
    /// Domains represent different classification categories like:
    /// - INTEL (Intelligence)
    /// - CYBER (Cybersecurity)
    /// - OPERATIONS (Military Operations)
    /// - LOGISTICS (Supply Chain)
    /// etc.
    pub async fn metadata_by_domain(
        &self,
        _context: &Context<'_>,
        domain: String,
    ) -> Result<Vec<Metadata>> {
        Metadata::get_by_domain(domain)
    }

    /// Returns all data object IDs that belong to a specific domain
    ///
    /// Useful for bulk operations or filtering data objects by domain.
    pub async fn data_object_ids_by_domain(
        &self,
        _context: &Context<'_>,
        domain: String,
    ) -> Result<Vec<Uuid>> {
        Metadata::get_data_object_ids_by_domain(domain)
    }

    /// Returns all data objects for a specific domain (with full data object details)
    ///
    /// This combines metadata domain filtering with complete data object information.
    pub async fn data_objects_by_domain(
        &self,
        _context: &Context<'_>,
        domain: String,
    ) -> Result<Vec<DataObject>> {
        let data_object_ids = Metadata::get_data_object_ids_by_domain(domain)?;
        DataObject::get_by_ids(data_object_ids)
    }

    /// Returns vector of all metadata records
    pub async fn metadata(&self, _context: &Context<'_>) -> Result<Vec<Metadata>> {
        Metadata::get_all()
    }

    /// Returns a limited number of metadata records
    pub async fn metadata_count_query(
        &self,
        _context: &Context<'_>,
        count: i64,
    ) -> Result<Vec<Metadata>> {
        let all_metadata = Metadata::get_all()?;
        Ok(all_metadata.into_iter().take(count as usize).collect())
    }

    /// Returns count of metadata records grouped by domain
    ///
    /// This provides a summary showing each unique domain and how many
    /// metadata records exist for that domain. Useful for understanding
    /// the distribution of classified data across different domains.
    pub async fn metadata_counts_by_domain(
        &self,
        _context: &Context<'_>,
    ) -> Result<Vec<DomainCount>> {
        Metadata::get_counts_by_domain()
    }
}
