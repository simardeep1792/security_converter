use async_graphql::*;
use uuid::Uuid;

use crate::models::{Authority, NewAuthority};
use crate::common_utils::{UserRole, is_admin, RoleGuard};

#[derive(Default)]
pub struct AuthorityMutation;

#[Object]
impl AuthorityMutation {
    #[graphql(
        name = "createAuthority",
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    /// Creates a new authority. Requires admin role.
    pub async fn create_authority(
        &self,
        _context: &Context<'_>,
        authority_data: NewAuthority,
    ) -> Result<Authority> {
        Authority::create(&authority_data)
    }

    #[graphql(
        name = "updateAuthority",
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    /// Updates an existing authority. Requires admin role.
    pub async fn update_authority(
        &self,
        _context: &Context<'_>,
        id: Uuid,
        authority_data: AuthorityUpdate,
    ) -> Result<Authority> {
        let mut authority = Authority::get_by_id(&id)?;

        if let Some(name) = authority_data.name {
            authority.name = name;
        }
        if let Some(email) = authority_data.email {
            authority.email = email;
        }
        if let Some(phone) = authority_data.phone {
            authority.phone = phone;
        }
        if let Some(expires_at) = authority_data.expires_at {
            authority.expires_at = expires_at;
        }

        authority.update()
    }
}

#[derive(Debug, InputObject)]
pub struct AuthorityUpdate {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub expires_at: Option<Option<chrono::NaiveDateTime>>,
}
