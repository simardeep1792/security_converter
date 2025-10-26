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

use crate::models::{Authority, ClassificationSchema, ConversionResponse,
    DataObject, InsertableConversionResponse, InsertableDataObject, InsertableMetadata, 
    Metadata, User};
use std::collections::HashMap;

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
    pub source_nation_classification: String,
    pub source_nation_code: String,
    pub target_nation_codes: Vec<Option<String>>, // At least one required, validated at creation
    //pub context_group: Option<String>, // Used for sending only to certain groups, missions or lists.
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub completed_at: Option<NaiveDateTime>,
}

/// The JSON formatted data payload submitted to the API that triggers
/// a security classification conversion
#[derive(Debug, Serialize, Deserialize, Clone, InputObject)]
#[graphql(name = "ConversionRequestInput")]
pub struct InsertableConversionRequest {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub data_object: InsertableDataObject,
    pub metadata: InsertableMetadata,
    pub source_nation_classification: String,
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
    pub async fn data_object(&self) -> Result<crate::models::data_object::DataObjectGraphQL> {
        let obj = DataObject::get_by_id(&self.data_object_id)?;
        Ok(obj.into())
    }

    /// Get the metadata for the data object
    pub async fn metadata(&self) -> Result<crate::models::metadata::MetadataGraphQL> {
        let meta = Metadata::get_by_data_object_id(&self.data_object_id)?;
        Ok(meta.into())
    }

    pub async fn conversion_response(&self) -> Result<ConversionResponse> {
        ConversionResponse::get_by_conversion_request_id(&self.id)
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
        let insertable_data_object = InsertableDataObject {
            title: payload.data_object.title.clone(),
            description: payload.data_object.description.clone(),
        };

        let new_data_object = insertable_data_object.to_new_data_object(payload.user_id);
        let data_object = DataObject::create(&new_data_object)?;

        // TODO: Handle if data object already exists

        // Step 2: Create the Metadata with the generated DataObject ID
        let insertable_metadata = InsertableMetadata {

            // Global Identifier
            identifier: payload.metadata.identifier.clone(),
            
            // Authorization Reference
            authorization_reference: payload.metadata.authorization_reference.clone(),
            authorization_reference_date: payload.metadata.authorization_reference_date,
            
            // Originator and Custodian
            originator_organization_id: payload.metadata.originator_organization_id,
            custodian_organization_id: payload.metadata.custodian_organization_id,
            
            // Format
            format: payload.metadata.format.clone(),
            format_size: payload.metadata.format_size,
            
            // Safeguarding and Securing
            security_classification: payload.metadata.security_classification.clone(),
            
            // Disclosure & Releasability
            releasable_to_countries: payload.metadata.releasable_to_countries.clone(),
            releasable_to_organizations: payload.metadata.releasable_to_organizations.clone(),
            releasable_to_categories: payload.metadata.releasable_to_categories.clone(),
            disclosure_category: payload.metadata.disclosure_category.clone(),
            
            // Handling Restrictions
            handling_restrictions: payload.metadata.handling_restrictions.clone(),
            handling_authority: payload.metadata.handling_authority.clone(),
            no_handling_restrictions: payload.metadata.no_handling_restrictions,
            
            // Legacy fields
            domain: payload.metadata.domain.clone(),
            tags: payload.metadata.tags.clone(),
        };

        let new_metadata = insertable_metadata.to_new_metadata(data_object.id);
        let _metadata = Metadata::create(&new_metadata)?;

        // Step 3: Create the ConversionRequest
        let new_request = NewConversionRequest {
            creator_id: payload.user_id,
            authority_id: payload.authority_id,
            data_object_id: data_object.id,
            source_nation_classification: payload.source_nation_classification.clone(),
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

    /// Get conversion request by data object ID
    pub fn get_by_data_object_id(data_object_id: &Uuid) -> Result<Self> {
        let mut conn = connection()?;
        let res = conversion_requests::table
            .filter(conversion_requests::data_object_id.eq(data_object_id))
            .first(&mut conn)?;
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

    /// Get all conversion requests by target nation code (checks if nation code is in target_nation_codes array)
    pub fn get_by_target_nation_code(nation_code: &str) -> Result<Vec<Self>> {

        let mut conn = connection()?;
        let res = conversion_requests::table
            .filter(conversion_requests::target_nation_codes.contains(vec![nation_code]))
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

    /// Process this conversion request and generate a ConversionResponse
    ///
    /// This performs the complete two-step conversion:
    /// 1. Source Nation Classification → NATO Standard
    /// 2. NATO Standard → Target Nation Classifications
    ///
    /// # Returns
    /// A `ConversionResponse` containing the NATO equivalent and all target nation classifications
    ///
    /// # Errors
    /// Returns an error if:
    /// - Source or target nation schemas are not found
    /// - Classification schemas have expired
    /// - Classification levels are invalid
    pub fn process_and_convert(&mut self) -> Result<ConversionResponse> {
        // Step 1: Get the source nation's classification schema
        let source_schema = ClassificationSchema::get_latest_by_nation_code(&self.source_nation_code)?;

        // Verify the schema is still valid
        if !source_schema.is_valid() {
            return Err(Error::new(format!(
                "Classification schema for nation {} has expired",
                self.source_nation_code
            )));
        }

        // Step 2: Convert source classification to NATO equivalent
        let nato_equivalent = source_schema.to_nato(&self.source_nation_classification)?;

        // Step 3: Convert NATO to each target nation classification
        let mut target_classifications = HashMap::new();

        for target_schema in ClassificationSchema::get_latest_by_nation_codes(&self.target_nation_codes)? {

            // Verify target schema is valid
            if !target_schema.is_valid() {
                return Err(Error::new(format!(
                    "Classification schema for target nation {} has expired",
                    target_schema.nation_code
                )));
            }

            let target_classification = target_schema.from_nato(&nato_equivalent)?;
            target_classifications.insert(target_schema.nation_code, target_classification);
        }

        // Step 4: Create the ConversionResponse
        let response_payload = InsertableConversionResponse {
            conversion_request_id: self.id,
            subject_data_id: self.data_object_id,
            nato_equivalent,
            target_nation_classifications: target_classifications,
        };

        let response = ConversionResponse::create(&response_payload);

        // Step 5: Update the Request to complete
        let _ = self.mark_completed().expect("Unable to mark request completed");

        response
    }
}

/// Standalone function to convert classifications between nations using NATO as the intermediary
///
/// This is the core conversion logic that implements the "Rosetta Stone" pattern:
/// Source Nation → NATO Standard → Target Nation(s)
///
/// # Arguments
/// * `source_nation_code` - The ISO 3166-1 alpha-3 code of the source nation (e.g., "USA", "GBR")
/// * `source_classification` - The classification level in the source nation's terminology
/// * `target_nation_codes` - Vector of target nation codes to convert to
///
/// # Returns
/// * `nato_equivalent` - The NATO classification level
/// * `target_classifications` - HashMap mapping nation codes to their equivalent classifications
///
/// # Example
/// ```
/// let (nato, targets) = convert_classification(
///     "USA",
///     "SECRET",
///     vec!["GBR".to_string(), "FRA".to_string()]
/// )?;
/// // nato == "NATO SECRET"
/// // targets == {"GBR": "SECRET", "FRA": "SECRET DÉFENSE"}
/// ```
pub fn convert_classification(
    source_nation_code: &str,
    source_classification: &str,
    target_nation_codes: Vec<String>,
) -> Result<(String, HashMap<String, String>)> {
    // Step 1: Get source nation's classification schema
    let source_schema = ClassificationSchema::get_latest_by_nation_code(
        &source_nation_code.to_string()
    )?;

    // Verify the schema is still valid
    if !source_schema.is_valid() {
        return Err(Error::new(format!(
            "Classification schema for nation {} has expired",
            source_nation_code
        )));
    }

    // Step 2: Convert source classification to NATO equivalent
    let nato_equivalent = source_schema.to_nato(source_classification)?;

    // Step 3: Convert NATO to each target nation classification
    let mut target_classifications = HashMap::new();

    for target_code in target_nation_codes {
        let target_schema = ClassificationSchema::get_latest_by_nation_code(&target_code)?;

        // Verify target schema is valid
        if !target_schema.is_valid() {
            return Err(Error::new(format!(
                "Classification schema for target nation {} has expired",
                target_code
            )));
        }

        let target_classification = target_schema.from_nato(&nato_equivalent)?;
        target_classifications.insert(target_code, target_classification);
    }

    Ok((nato_equivalent, target_classifications))
}

/// Internal struct for inserting conversion requests into the database
/// This is created internally after DataObject and Metadata have been created
#[derive(Debug, Clone, Deserialize, Serialize, Insertable)]
#[diesel(table_name = conversion_requests)]
struct NewConversionRequest {
    pub creator_id: Uuid,
    pub authority_id: Uuid,
    pub data_object_id: Uuid,
    pub source_nation_classification: String,
    pub source_nation_code: String,
    pub target_nation_codes: Vec<String>,
}
