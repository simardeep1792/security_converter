use std::fmt::Debug;

use async_graphql::*;
use chrono::prelude::*;
use diesel::{self, ExpressionMethods, Insertable, PgTextExpressionMethods, Queryable};
use diesel::{QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{Metadata, User, ConversionRequest, ConversionResponse};
use crate::{database, schema::*};

#[derive(
    Debug, Clone, Deserialize, Serialize, Queryable, Insertable, AsChangeset, SimpleObject,
)]
#[graphql(complex)]
#[diesel(table_name = data_objects)]
pub struct DataObject {
    pub id: Uuid,
    pub creator_id: Uuid, // User
    pub title: String,
    pub description: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// GraphQL implementation
#[ComplexObject]
impl DataObject {
    pub async fn creator(&self) -> Result<User> {
        User::get_by_id(&self.creator_id)
    }

    pub async fn metadata(&self) -> Result<Metadata> {
        Metadata::get_by_data_object_id(&self.id)
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

        let res = data_objects::table
            .filter(data_objects::title.eq(&data_object.title))
            .filter(data_objects::creator_id.eq(&data_object.creator_id))
            .distinct()
            .first(&mut conn);

        let data_object = match res {
            Ok(d) => d,
            Err(e) => {
                // DataObject not found
                println!("{:?}", e);
                let d = DataObject::create(data_object).expect("Unable to create data_object");
                d
            }
        };
        Ok(data_object)
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
        let search_pattern = format!("%{}%", title);
        let res = data_objects::table
            .filter(data_objects::title.ilike(search_pattern))
            .load::<DataObject>(&mut conn)?;
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

#[derive(Debug, Clone, Deserialize, Serialize, Insertable, InputObject)]
#[diesel(table_name = data_objects)]
pub struct NewDataObject {
    pub creator_id: Uuid, // User
    pub title: String,
    pub description: String,
}

impl NewDataObject {
    pub fn new(creator_id: Uuid, title: String, description: String) -> Self {
        NewDataObject {
            creator_id,
            title,
            description,
        }
    }
}

/// A lightweight struct to accept JSON formatted data from a ConversionRequest
/// needed to create a NewDataObject
#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
#[graphql(name = "DataObjectInput")]
pub struct InsertableDataObject {
    pub title: String,
    pub description: String,
}
