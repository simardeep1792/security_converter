use std::fmt::Debug;

use async_graphql::*;
use chrono::prelude::*;
use diesel::{self, ExpressionMethods, Insertable, Queryable};
use diesel::{QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::encryption::EncryptedString;
use crate::models::{Metadata, User, ConversionRequest, ConversionResponse};
use crate::{database, schema::*};

/// Database model with encrypted fields
/// This is the actual struct that interacts with the database
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = data_objects)]
pub struct DataObject {
    pub id: Uuid,
    pub creator_id: Uuid, // User
    pub title: EncryptedString,      // 🔒 ENCRYPTED in database
    pub description: EncryptedString, // 🔒 ENCRYPTED in database
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// GraphQL model with decrypted String fields
/// This is what gets exposed to the GraphQL API
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
#[graphql(complex)]
pub struct DataObjectGraphQL {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub title: String,        // Decrypted for GraphQL
    pub description: String,   // Decrypted for GraphQL
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// Convert database model to GraphQL model
impl From<DataObject> for DataObjectGraphQL {
    fn from(obj: DataObject) -> Self {
        Self {
            id: obj.id,
            creator_id: obj.creator_id,
            title: obj.title.into_string(),
            description: obj.description.into_string(),
            created_at: obj.created_at,
            updated_at: obj.updated_at,
        }
    }
}

// GraphQL implementation for complex fields
#[ComplexObject]
impl DataObjectGraphQL {
    pub async fn creator(&self, ctx: &Context<'_>) -> Result<User> {
        let loaders = ctx.data::<crate::graphql::Loaders>()?;
        let user = loaders.user_loader.load(self.creator_id).await;
        Ok(user)
    }

    pub async fn metadata(&self) -> Result<crate::models::metadata::MetadataGraphQL> {
        let meta = Metadata::get_by_data_object_id(&self.id)?;
        Ok(meta.into())
    }

    pub async fn conversion_request(&self) -> Result<ConversionRequest> {
        ConversionRequest::get_by_data_object_id(&self.id)
    }

    pub async fn conversion_response(&self) -> Result<ConversionResponse> {
        ConversionResponse::get_by_data_object_id(&self.id)
    }
}

// Non Graphql
impl DataObject {
    pub fn create(data_object: &NewDataObject) -> Result<Self> {
        let mut conn = database::connection()?;

        let res = diesel::insert_into(data_objects::table)
            .values(data_object)
            .get_result(&mut conn)?;

        Ok(res)
    }

    pub fn get_or_create(data_object: &NewDataObject) -> Result<Self> {
        let mut conn = database::connection()?;

        // Note: Cannot filter by encrypted fields directly
        // This now loads all objects by creator and checks in-memory
        let existing_objects = data_objects::table
            .filter(data_objects::creator_id.eq(&data_object.creator_id))
            .load::<DataObject>(&mut conn)?;

        // Check if any existing object matches the title (after decryption)
        for obj in existing_objects {
            if obj.title.as_str() == data_object.title.as_str() {
                return Ok(obj);
            }
        }

        // Not found, create new
        let d = DataObject::create(data_object)?;
        Ok(d)
    }

    pub fn get_all() -> Result<Vec<Self>> {
        let mut conn = database::connection()?;
        let res = data_objects::table.load::<DataObject>(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_id(id: &Uuid) -> Result<Self> {
        let mut conn = database::connection()?;
        let res = data_objects::table
            .filter(data_objects::id.eq(id))
            .first(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_ids(ids: Vec<Uuid>) -> Result<Vec<Self>> {
        let mut conn = database::connection()?;
        let res = data_objects::table
            .filter(data_objects::id.eq_any(ids))
            .load::<DataObject>(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_creator_id(creator_id: Uuid) -> Result<Vec<Self>> {
        let mut conn = database::connection()?;
        let res = data_objects::table
            .filter(data_objects::creator_id.eq(creator_id))
            .load::<DataObject>(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_title(title: &String) -> Result<Vec<Self>> {
        let mut conn = database::connection()?;

        // Note: Cannot search encrypted fields directly in database
        // Load all records and filter in-memory (acceptable for small datasets)
        // For large datasets, consider implementing a hash-based search index
        let all_objects = data_objects::table.load::<DataObject>(&mut conn)?;

        let search_lower = title.to_lowercase();
        let res: Vec<Self> = all_objects
            .into_iter()
            .filter(|obj| obj.title.as_str().to_lowercase().contains(&search_lower))
            .collect();

        Ok(res)
    }

    pub fn get_count() -> Result<i64> {
        let mut conn = database::connection()?;

        let res = data_objects::table.count().get_result(&mut conn)?;

        Ok(res)
    }

    // TODO: get_by_metadata_tags

    pub fn update(&self) -> Result<Self> {
        let mut conn = database::connection()?;

        let res = diesel::update(data_objects::table)
            .filter(data_objects::id.eq(&self.id))
            .set(self)
            .get_result(&mut conn)?;

        Ok(res)
    }
}

/// Database insertable model with encrypted fields
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = data_objects)]
pub struct NewDataObject {
    pub creator_id: Uuid, // User
    pub title: EncryptedString,      // 🔒 Will be encrypted on insert
    pub description: EncryptedString, // 🔒 Will be encrypted on insert
}

impl NewDataObject {
    pub fn new(creator_id: Uuid, title: String, description: String) -> Self {
        NewDataObject {
            creator_id,
            title: EncryptedString::from(title),
            description: EncryptedString::from(description),
        }
    }

    /// Create from plain strings (convenience method)
    pub fn from_strings(creator_id: Uuid, title: String, description: String) -> Self {
        Self::new(creator_id, title, description)
    }

    /// Create from EncryptedStrings (for internal use)
    pub fn from_encrypted(creator_id: Uuid, title: EncryptedString, description: EncryptedString) -> Self {
        NewDataObject {
            creator_id,
            title,
            description,
        }
    }
}

/// A lightweight struct to accept JSON formatted data from a ConversionRequest
/// needed to create a NewDataObject
/// GraphQL input type accepts plain String (will be encrypted internally)
#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
#[graphql(name = "DataObjectInput")]
pub struct InsertableDataObject {
    pub title: String,        // GraphQL input as plain String
    pub description: String,   // GraphQL input as plain String
}

impl InsertableDataObject {
    /// Convert to NewDataObject with encrypted fields
    pub fn to_new_data_object(&self, creator_id: Uuid) -> NewDataObject {
        NewDataObject::new(creator_id, self.title.clone(), self.description.clone())
    }
}
