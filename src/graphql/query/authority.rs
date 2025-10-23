use async_graphql::*;

use crate::models::Authority;
use uuid::Uuid;

//use crate::common_utils::{RoleGuard, is_admin, UserRole};

#[derive(Default)]
pub struct AuthorityQuery;

#[Object]
impl AuthorityQuery {
    /// Returns count of Authorities in the system
    pub async fn authority_count(&self, _context: &Context<'_>) -> Result<i64> {
        let authorities = Authority::get_all()?;
        Ok(authorities.len() as i64)
    }

    /// Returns an authority by its Uuid
    pub async fn authority_by_id(&self, _context: &Context<'_>, id: Uuid) -> Result<Authority> {
        Authority::get_by_id(&id)
    }

    /// Returns authorities by creator ID
    pub async fn authorities_by_creator_id(
        &self,
        _context: &Context<'_>,
        creator_id: Uuid,
    ) -> Result<Vec<Authority>> {
        Authority::get_by_creator_id(creator_id)
    }

    /// Returns authorities by nation ID
    pub async fn authorities_by_nation_code(
        &self,
        _context: &Context<'_>,
        nation_code: String,
    ) -> Result<Vec<Authority>> {
        Authority::get_by_nation_code(&nation_code)
    }

    /// Returns vector of all authorities
    pub async fn authorities(&self, _context: &Context<'_>) -> Result<Vec<Authority>> {
        Authority::get_all()
    }
}
