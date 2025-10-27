use dataloader::{BatchFn, cached::Loader};
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::{User, Authority, DataObject, DataObjectGraphQL};

/// UserBatchLoader - Batches User::get_by_id requests into a single query
#[derive(Clone)]
pub struct UserBatchLoader;

#[async_trait::async_trait]
impl BatchFn<Uuid, User> for UserBatchLoader {
    async fn load(&mut self, keys: &[Uuid]) -> HashMap<Uuid, User> {
        match User::get_by_ids(keys.to_vec()) {
            Ok(users) => users
                .into_iter()
                .map(|user| (user.id, user))
                .collect(),
            Err(_) => HashMap::new(),
        }
    }
}

/// AuthorityBatchLoader - Batches Authority::get_by_id requests into a single query
#[derive(Clone)]
pub struct AuthorityBatchLoader;

#[async_trait::async_trait]
impl BatchFn<Uuid, Authority> for AuthorityBatchLoader {
    async fn load(&mut self, keys: &[Uuid]) -> HashMap<Uuid, Authority> {
        match Authority::get_by_ids(keys.to_vec()) {
            Ok(authorities) => authorities
                .into_iter()
                .map(|authority| (authority.id, authority))
                .collect(),
            Err(_) => HashMap::new(),
        }
    }
}

/// DataObjectBatchLoader - Batches DataObject::get_by_id requests into a single query
/// Returns DataObjectGraphQL (decrypted) for GraphQL API use
#[derive(Clone)]
pub struct DataObjectBatchLoader;

#[async_trait::async_trait]
impl BatchFn<Uuid, DataObjectGraphQL> for DataObjectBatchLoader {
    async fn load(&mut self, keys: &[Uuid]) -> HashMap<Uuid, DataObjectGraphQL> {
        match DataObject::get_by_ids(keys.to_vec()) {
            Ok(data_objects) => data_objects
                .into_iter()
                .map(|obj| {
                    let id = obj.id;
                    (id, obj.into()) // Convert DataObject to DataObjectGraphQL
                })
                .collect(),
            Err(_) => HashMap::new(),
        }
    }
}

/// Loaders struct that holds all DataLoaders
/// This will be stored in the GraphQL context
pub struct Loaders {
    pub user_loader: Loader<Uuid, User, UserBatchLoader>,
    pub authority_loader: Loader<Uuid, Authority, AuthorityBatchLoader>,
    pub data_object_loader: Loader<Uuid, DataObjectGraphQL, DataObjectBatchLoader>,
}

impl Loaders {
    /// Create a new Loaders instance with all DataLoaders initialized
    pub fn new() -> Self {
        Self {
            user_loader: Loader::new(UserBatchLoader),
            authority_loader: Loader::new(AuthorityBatchLoader),
            data_object_loader: Loader::new(DataObjectBatchLoader),
        }
    }
}

impl Default for Loaders {
    fn default() -> Self {
        Self::new()
    }
}
