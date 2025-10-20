use std::collections::HashMap;
use std::fmt::Debug;

use async_graphql::*;
use chrono::prelude::*;
use diesel::prelude::*;
use diesel::{
    self, ExpressionMethods, Insertable, Queryable,
};
use diesel::{QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::database::connection;

use crate::{database, schema::*};

use crate::models::{ConversionRequest, DataObject};

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
#[diesel(belongs_to(ConversionRequest, foreign_key = conversion_request_id))]
#[diesel(belongs_to(DataObject, foreign_key = subject_data_id))]
#[diesel(table_name = conversion_responses)]
#[graphql(complex)]
/// A response from the security classification conversion middleware.
/// This represents the result of processing a ConversionRequest, containing the
/// NATO equivalent classification and the target nation classifications.
pub struct ConversionResponse {
    pub id: Uuid,
    pub conversion_request_id: Uuid, // ConversionRequest that generated this response
    pub subject_data_id: Uuid, // DataObject being classified
    pub nato_equivalent: String, // NATO classification level
    pub target_nation_classifications: JsonValue, // HashMap<NationCode, String> stored as JSONB
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub expires_at: Option<NaiveDateTime>,
}

/// The data payload for creating a conversion response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InsertableConversionResponse {
    pub conversion_request_id: Uuid,
    pub subject_data_id: Uuid,
    pub nato_equivalent: String,
    pub target_nation_classifications: HashMap<String, String>,
}

// GraphQL Complex Object implementation
#[ComplexObject]
impl ConversionResponse {
    /// Get the conversion request that generated this response
    pub async fn conversion_request(&self) -> Result<ConversionRequest> {
        ConversionRequest::get_by_id(&self.conversion_request_id)
    }

    /// Get the data object that was classified
    pub async fn subject_data(&self) -> Result<crate::models::data_object::DataObjectGraphQL> {
        let obj = DataObject::get_by_id(&self.subject_data_id)?;
        Ok(obj.into())
    }

    /// Get the target nation classifications as a HashMap
    pub async fn target_classifications(&self) -> Result<HashMap<String, String>> {
        serde_json::from_value(self.target_nation_classifications.clone())
            .map_err(|e| Error::new(format!("Failed to deserialize target classifications: {}", e)))
    }

    /// Check if this conversion response has expired
    pub async fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            expires_at < Utc::now().naive_utc()
        } else {
            false
        }
    }
}

// Non GraphQL implementation
impl ConversionResponse {
    /// Create a new conversion response from a processed conversion request
    ///
    /// This is called after the classification conversion logic has determined:
    /// - The NATO equivalent classification
    /// - The target nation classifications for each requested nation
    pub fn create(payload: &InsertableConversionResponse) -> Result<ConversionResponse> {
        let mut conn = connection()?;

        let new_response = NewConversionResponse {
            conversion_request_id: payload.conversion_request_id,
            subject_data_id: payload.subject_data_id,
            nato_equivalent: payload.nato_equivalent.clone(),
            target_nation_classifications: serde_json::to_value(&payload.target_nation_classifications)
                .map_err(|e| Error::new(format!("Failed to serialize target classifications: {}", e)))?,
        };

        let conversion_response = diesel::insert_into(conversion_responses::table)
            .values(&new_response)
            .get_result(&mut conn)?;

        Ok(conversion_response)
    }

    /// Get all conversion responses
    pub fn get_all() -> Result<Vec<Self>> {
        let mut conn = database::connection()?;
        let res = conversion_responses::table.load::<ConversionResponse>(&mut conn)?;
        Ok(res)
    }

    /// Get a limited number of conversion responses
    pub fn get_count(count: i64) -> Result<Vec<Self>> {
        let mut conn = database::connection()?;
        let res = conversion_responses::table
            .limit(count)
            .load::<ConversionResponse>(&mut conn)?;
        Ok(res)
    }

    /// Get a conversion response by ID
    pub fn get_by_id(id: &Uuid) -> Result<Self> {
        let mut conn = database::connection()?;
        let res = conversion_responses::table
            .filter(conversion_responses::id.eq(id))
            .first(&mut conn)?;
        Ok(res)
    }

    /// Get conversion responses for a specific conversion request
    pub fn get_by_conversion_request_id(request_id: &Uuid) -> Result<Self> {
        let mut conn = connection()?;
        let res = conversion_responses::table
            .filter(conversion_responses::conversion_request_id.eq(request_id))
            .first(&mut conn)?;
        Ok(res)
    }

    /// Get conversion response for a specific data object
    pub fn get_by_data_object_id(data_object_id: &Uuid) -> Result<Self> {
        let mut conn = connection()?;
        let res = conversion_responses::table
            .filter(conversion_responses::subject_data_id.eq(data_object_id))
            .first(&mut conn)?;
        Ok(res)
    }

    /// Get all conversion responses by NATO equivalent classification
    pub fn get_by_nato_equivalent(nato_classification: &str) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let res = conversion_responses::table
            .filter(conversion_responses::nato_equivalent.eq(nato_classification))
            .load::<ConversionResponse>(&mut conn)?;
        Ok(res)
    }

    /// Get all non-expired conversion responses
    pub fn get_active() -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let now = Utc::now().naive_utc();
        let res = conversion_responses::table
            .filter(
                conversion_responses::expires_at.is_null()
                    .or(conversion_responses::expires_at.gt(now))
            )
            .load::<ConversionResponse>(&mut conn)?;
        Ok(res)
    }

    /// Get all expired conversion responses
    pub fn get_expired() -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let now = Utc::now().naive_utc();
        let res = conversion_responses::table
            .filter(conversion_responses::expires_at.is_not_null())
            .filter(conversion_responses::expires_at.lt(now))
            .load::<ConversionResponse>(&mut conn)?;
        Ok(res)
    }

    /// Update a conversion response with changed data
    pub fn update(&self) -> Result<Self> {
        let mut conn = database::connection()?;
        let res = diesel::update(conversion_responses::table)
            .filter(conversion_responses::id.eq(&self.id))
            .set(self)
            .get_result(&mut conn)?;
        Ok(res)
    }

    /// Delete a conversion response
    pub fn delete(&self) -> Result<usize> {
        let mut conn = database::connection()?;
        let res = diesel::delete(conversion_responses::table)
            .filter(conversion_responses::id.eq(&self.id))
            .execute(&mut conn)?;
        Ok(res)
    }
}

/// Internal struct for inserting conversion responses into the database
#[derive(Debug, Clone, Deserialize, Serialize, Insertable)]
#[diesel(table_name = conversion_responses)]
struct NewConversionResponse {
    pub conversion_request_id: Uuid,
    pub subject_data_id: Uuid,
    pub nato_equivalent: String,
    pub target_nation_classifications: JsonValue,
}
