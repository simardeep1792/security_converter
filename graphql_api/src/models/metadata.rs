use std::fmt::Debug;
use std::collections::HashMap;

use async_graphql::*;
use chrono::prelude::*;
use diesel::{self, ExpressionMethods, Insertable, Queryable};
use diesel::{QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::DataObject;
use crate::{database, schema::*};

/// Represents the count of metadata records for a specific domain
#[derive(Debug, Clone, Deserialize, Serialize, SimpleObject)]
pub struct DomainCount {
    pub domain: String,
    pub count: i64,
}

#[derive(
    Debug, Clone, Deserialize, Serialize, Queryable, Insertable, AsChangeset, SimpleObject,
)]
#[graphql(complex)]
#[diesel(table_name = metadata)]
pub struct Metadata {
    pub id: Uuid,
    pub data_object_id: Uuid,
    pub domain: String,
    pub tags: Vec<Option<String>>, // PostgreSQL TEXT[] array
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// GraphQL implementation
#[ComplexObject]
impl Metadata {
    pub async fn get_data_object(&self) -> Result<DataObject> {
        DataObject::get_by_id(&self.data_object_id)
    }
}

// Non Graphql
impl Metadata {
    pub fn create(metadata: &NewMetadata) -> Result<Self> {
        let mut conn = database::connection()?;

        let res = diesel::insert_into(metadata::table)
            .values(metadata)
            .get_result(&mut conn)?;

        Ok(res)
    }

    pub fn get_or_create(metadata: &NewMetadata) -> Result<Self> {
        let mut conn = database::connection()?;

        let res = metadata::table
            .filter(metadata::domain.eq(&metadata.domain))
            .distinct()
            .first(&mut conn);

        let metadata = match res {
            Ok(m) => m,
            Err(e) => {
                // Metadata not found
                println!("{:?}", e);
                let m = Metadata::create(metadata).expect("Unable to create metadata");
                m
            }
        };
        Ok(metadata)
    }

    pub fn get_all() -> Result<Vec<Self>> {
        let mut conn = database::connection()?;
        let res = metadata::table.load::<Metadata>(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_id(id: &Uuid) -> Result<Self> {
        let mut conn = database::connection()?;
        let res = metadata::table
            .filter(metadata::id.eq(id))
            .first(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_domain(domain: String) -> Result<Vec<Self>> {
        let mut conn = database::connection()?;
        let res = metadata::table
            .filter(metadata::domain.eq(domain))
            .load::<Metadata>(&mut conn)?;
        Ok(res)
    }

    pub fn get_data_object_ids_by_domain(domain: String) -> Result<Vec<Uuid>> {
        let mut conn = database::connection()?;
        let res = metadata::table
            .filter(metadata::domain.eq(domain))
            .select((metadata::data_object_id))
            .load::<Uuid>(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_data_object_id(data_object_id: &Uuid) -> Result<Self> {
        let mut conn = database::connection()?;
        let res = metadata::table
            .filter(metadata::data_object_id.eq(data_object_id))
            .first(&mut conn)?;
        Ok(res)
    }

    pub fn get_counts_by_domain() -> Result<Vec<DomainCount>> {
        let mut conn = database::connection()?;
        let all_metadata = metadata::table.load::<Metadata>(&mut conn)?;

        let mut domain_counts: HashMap<String, i64> = HashMap::new();

        for metadata in all_metadata {
            *domain_counts.entry(metadata.domain.clone()).or_insert(0) += 1;
        }

        let result: Vec<DomainCount> = domain_counts
            .into_iter()
            .map(|(domain, count)| DomainCount { domain, count })
            .collect();

        Ok(result)
    }

    pub fn update(&self) -> Result<Self> {
        let mut conn = database::connection()?;

        let res = diesel::update(metadata::table)
            .filter(metadata::id.eq(&self.id))
            .set(self)
            .get_result(&mut conn)?;

        Ok(res)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable, InputObject)]
#[diesel(table_name = metadata)]
pub struct NewMetadata {
    pub data_object_id: Uuid,
    pub domain: String,
    pub tags: Vec<Option<String>>,
}

impl NewMetadata {
    pub fn new(data_object_id: Uuid, domain: String, tags: Vec<Option<String>>) -> Self {
        NewMetadata {
            data_object_id,
            domain,
            tags,
        }
    }
}

/// A light struct to accept the JSON formatted Metadata included with
/// a ConversionRequest
#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
#[graphql(name = "MetadataInput")]
pub struct InsertableMetadata {
    pub domain: String,
    pub tags: Vec<Option<String>>,
}
