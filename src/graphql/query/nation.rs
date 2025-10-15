use async_graphql::*;

use crate::models::Nation;
use uuid::Uuid;

//use crate::common_utils::{RoleGuard, is_admin, UserRole};

#[derive(Default)]
pub struct NationQuery;

#[Object]
impl NationQuery {
    /// Returns count of Nations in the system
    pub async fn nation_count(&self, _context: &Context<'_>) -> Result<i64> {
        let nations = Nation::get_all()?;
        Ok(nations.len() as i64)
    }

    /// Returns a nation by its Uuid
    pub async fn nation_by_id(&self, _context: &Context<'_>, id: Uuid) -> Result<Nation> {
        Nation::get_by_id(&id)
    }

    /// Returns a nation by its nation code
    pub async fn nation_by_code(&self, _context: &Context<'_>, nation_code: String) -> Result<Nation> {
        Nation::get_by_code(&nation_code)
    }

    /// Returns nations by creator ID
    pub async fn nations_by_creator_id(
        &self,
        _context: &Context<'_>,
        creator_id: Uuid,
    ) -> Result<Vec<Nation>> {
        Nation::get_by_creator_id(creator_id)
    }

    /// Returns vector of all nations
    pub async fn nations(&self, _context: &Context<'_>) -> Result<Vec<Nation>> {
        Nation::get_all()
    }
}
