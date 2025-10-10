use std::fmt::Debug;

use async_graphql::*;
use chrono::prelude::*;
use diesel::prelude::*;
use diesel::{
    self, ExpressionMethods, Insertable, Queryable,
};
use diesel::{QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::connection;

use crate::{database, schema::*};

use crate::models::{Authority, DataObject, InsertableDataObject, InsertableMetadata, Metadata, NewDataObject, NewMetadata, User};

#[derive(
    Debug,
    Clone,
    Deserialize,
    Serialize,
    Queryable,
    Identifiable,
    Insertable,
    AsChangeset,
    SimpleObject,
    Associations,
)]
#[diesel(belongs_to(User, foreign_key = creator_id))]
#[diesel(belongs_to(Authority, foreign_key = authority_id))]
#[diesel(belongs_to(DataObject, foreign_key = data_object_id))]
#[diesel(table_name = conversion_requests)]
#[graphql(complex)]
/// A request for security classification conversion to the middleware.
/// This would be a JSON data package sent to the middleware by an validated 
/// user who is part of an accredited authority and should include the information
/// necessary to generate conversion response.
pub struct ConversionRequest {
    pub id: Uuid,
    pub creator_id: Uuid, // User
    pub authority_id: Uuid, // Authority requesting conversion
    pub data_object_id: Uuid,     // DataObject
    pub source_nation_code: String,
    pub target_nation_codes: Vec<Option<String>>, // At least one required, validated at creation
    //pub context_group: Option<String>, // Used for sending only to certain groups, missions or lists.
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub completed_at: Option<NaiveDateTime>,
}

/// The JSON formatted data payload submitted to the API that triggers
/// a security classification conversion
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InsertableConversionRequest {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub data_object: InsertableDataObject,
    pub metadata: InsertableMetadata,
    pub source_nation_code: String,
    pub target_nation_codes: Vec<String>,
}

// GraphQL Complex Object implementation
#[ComplexObject]
impl ConversionRequest {
    /// Get the user who created this conversion request
    pub async fn creator(&self) -> Result<User> {
        User::get_by_id(&self.creator_id)
    }

    /// Get the authority that is requesting this conversion
    pub async fn authority(&self) -> Result<Authority> {
        Authority::get_by_id(&self.authority_id)
    }

    /// Get the data object that is being converted
    pub async fn data_object(&self) -> Result<DataObject> {
        DataObject::get_by_id(&self.data_object_id)
    }

    /// Get the metadata for the data object
    pub async fn metadata(&self) -> Result<Metadata> {
        Metadata::get_by_data_object_id(&self.data_object_id)
    }

    /// Check if this conversion request has been completed
    pub async fn is_completed(&self) -> bool {
        self.completed_at.is_some()
    }
}

// Non GraphQL implementation
impl ConversionRequest {
    /// Process a conversion request payload by creating data objects, metadata, and the request itself
    /// This is the main entry point for handling incoming conversion requests
    ///
    /// Workflow:
    /// 1. Create DataObject from payload
    /// 2. Create Metadata with the new DataObject ID
    /// 3. Create ConversionRequest with all IDs
    /// 4. TODO: Trigger security classification conversion process
    pub fn process_payload(payload: &InsertableConversionRequest) -> Result<ConversionRequest> {
        let mut conn = connection()?;

        // Step 1: Create the DataObject
        let new_data_object = NewDataObject {
            creator_id: payload.user_id,
            title: payload.data_object.title.clone(),
            description: payload.data_object.description.clone(),
        };
        let data_object = DataObject::create(&new_data_object)?;

        // Step 2: Create the Metadata with the generated DataObject ID
        let new_metadata = NewMetadata {
            data_object_id: data_object.id,
            domain: payload.metadata.domain.clone(),
            tags: payload.metadata.tags.clone(),
        };
        let _metadata = Metadata::create(&new_metadata)?;

        // Step 3: Create the ConversionRequest
        let new_request = NewConversionRequest {
            creator_id: payload.user_id,
            authority_id: payload.authority_id,
            data_object_id: data_object.id,
            source_nation_code: payload.source_nation_code.clone(),
            target_nation_codes: payload.target_nation_codes.clone(),
        };

        let conversion_request = diesel::insert_into(conversion_requests::table)
            .values(&new_request)
            .get_result(&mut conn)?;

        // TODO: Trigger security classification conversion process here
        // This would involve:
        // - Looking up source nation classification schema
        // - Converting to NATO standard
        // - Converting from NATO to each target nation
        // - Generating response

        Ok(conversion_request)
    }

    /// Get all conversion requests
    pub fn get_all() -> Result<Vec<Self>> {
        let mut conn = database::connection()?;
        let res = conversion_requests::table.load::<ConversionRequest>(&mut conn)?;
        Ok(res)
    }

    /// Get a limited number of conversion requests
    pub fn get_count(count: i64) -> Result<Vec<Self>> {
        let mut conn = database::connection()?;
        let res = conversion_requests::table
            .limit(count)
            .load::<ConversionRequest>(&mut conn)?;
        Ok(res)
    }

    /// Get a conversion request by ID
    pub fn get_by_id(id: &Uuid) -> Result<Self> {
        let mut conn = database::connection()?;
        let res = conversion_requests::table
            .filter(conversion_requests::id.eq(id))
            .first(&mut conn)?;
        Ok(res)
    }

    /// Get all conversion requests by creator ID
    pub fn get_by_creator_id(creator_id: &Uuid) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let res = conversion_requests::table
            .filter(conversion_requests::creator_id.eq(creator_id))
            .load::<ConversionRequest>(&mut conn)?;
        Ok(res)
    }

    /// Get all conversion requests by authority ID
    pub fn get_by_authority_id(authority_id: &Uuid) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let res = conversion_requests::table
            .filter(conversion_requests::authority_id.eq(authority_id))
            .load::<ConversionRequest>(&mut conn)?;
        Ok(res)
    }

    /// Get all conversion requests by data object ID
    pub fn get_by_data_object_id(data_object_id: &Uuid) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let res = conversion_requests::table
            .filter(conversion_requests::data_object_id.eq(data_object_id))
            .load::<ConversionRequest>(&mut conn)?;
        Ok(res)
    }

    /// Get all conversion requests by source nation code
    pub fn get_by_source_nation_code(nation_code: &str) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let res = conversion_requests::table
            .filter(conversion_requests::source_nation_code.eq(nation_code))
            .load::<ConversionRequest>(&mut conn)?;
        Ok(res)
    }

    /// Get all pending (not completed) conversion requests
    pub fn get_pending() -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let res = conversion_requests::table
            .filter(conversion_requests::completed_at.is_null())
            .load::<ConversionRequest>(&mut conn)?;
        Ok(res)
    }

    /// Get all completed conversion requests
    pub fn get_completed() -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let res = conversion_requests::table
            .filter(conversion_requests::completed_at.is_not_null())
            .load::<ConversionRequest>(&mut conn)?;
        Ok(res)
    }

    /// Mark a conversion request as completed
    pub fn mark_completed(&mut self) -> Result<Self> {
        self.completed_at = Some(Utc::now().naive_utc());
        self.update()
    }

    /// Update a conversion request with changed data
    pub fn update(&self) -> Result<Self> {
        let mut conn = database::connection()?;
        let res = diesel::update(conversion_requests::table)
            .filter(conversion_requests::id.eq(&self.id))
            .set(self)
            .get_result(&mut conn)?;
        Ok(res)
    }

    /// Delete a conversion request
    pub fn delete(&self) -> Result<usize> {
        let mut conn = database::connection()?;
        let res = diesel::delete(conversion_requests::table)
            .filter(conversion_requests::id.eq(&self.id))
            .execute(&mut conn)?;
        Ok(res)
    }
}

/// Internal struct for inserting conversion requests into the database
/// This is created internally after DataObject and Metadata have been created
#[derive(Debug, Clone, Deserialize, Serialize, Insertable)]
#[diesel(table_name = conversion_requests)]
struct NewConversionRequest {
    pub creator_id: Uuid,
    pub authority_id: Uuid,
    pub data_object_id: Uuid,
    pub source_nation_code: String,
    pub target_nation_codes: Vec<String>,
}
