use std::fmt::Debug;

use async_graphql::*;
use chrono::prelude::*;
use diesel::{self, ExpressionMethods, Insertable, Queryable};
use diesel::{QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{Authority, ConversionRequest, User};
use crate::{database, schema::*};

#[derive(
    Debug, Clone, Deserialize, Serialize, Queryable, Insertable, AsChangeset, SimpleObject,
)]
#[graphql(complex)]
#[diesel(table_name = nations)]
#[diesel(belongs_to(User))]
pub struct Nation {
    pub id: Uuid,
    pub creator_id: Uuid, // User
    pub nation_code: String,
    pub nation_name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// GraphQL implementation
#[ComplexObject]
impl Nation {
    pub async fn creator(&self, ctx: &Context<'_>) -> Result<User> {
        let loaders = ctx.data::<crate::graphql::Loaders>()?;
        let user = loaders.user_loader.load(self.creator_id).await;
        Ok(user)
    }

    pub async fn authorities(&self) -> Result<Vec<Authority>> {
        Authority::get_by_nation_id(&self.id)
    }

    pub async fn conversion_requests(&self) -> Result<Vec<ConversionRequest>> {
        ConversionRequest::get_by_source_nation_code(&self.nation_code)
    }

    pub async fn inbound_conversion_requests(&self) -> Result<Vec<ConversionRequest>> {
        ConversionRequest::get_by_target_nation_code(&self.nation_code)
    }
}

// Non Graphql
impl Nation {
    pub fn create(nation: &NewNation) -> Result<Self> {
        let mut conn = database::connection()?;

        let res = diesel::insert_into(nations::table)
            .values(nation)
            .get_result(&mut conn)?;

        Ok(res)
    }

    pub fn get_or_create(nation: &NewNation) -> Result<Self> {
        let mut conn = database::connection()?;

        let res = nations::table
            .filter(nations::creator_id.eq(&nation.creator_id))
            .distinct()
            .first(&mut conn);

        let nation = match res {
            Ok(p) => p,
            Err(e) => {
                // Nation not found
                println!("{:?}", e);
                let p = Nation::create(nation).expect("Unable to create nation");
                p
            }
        };
        Ok(nation)
    }

    pub fn get_all() -> Result<Vec<Self>> {
        let mut conn = database::connection()?;
        let res = nations::table.load::<Nation>(&mut conn)?;
        Ok(res)
    }

    pub fn get_all_codes() -> Result<Vec<String>> {
        let mut conn = database::connection()?;
        let res = nations::table
            .select(nations::nation_code)
            .load::<String>(&mut conn)?;
        
        Ok(res)
    }

    pub fn get_by_id(id: &Uuid) -> Result<Self> {
        let mut conn = database::connection()?;
        let res = nations::table.filter(nations::id.eq(id)).first(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_creator_id(creator_id: Uuid) -> Result<Vec<Self>> {
        let mut conn = database::connection()?;
        let res = nations::table
            .filter(nations::creator_id.eq(creator_id))
            .load::<Nation>(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_code(nation_code: &String) -> Result<Self> {
        let mut conn = database::connection()?;
        let res = nations::table
            .filter(nations::nation_code.eq(nation_code))
            .first(&mut conn)?;
        Ok(res)
    }

    pub fn update(&self) -> Result<Self> {
        let mut conn = database::connection()?;

        let res = diesel::update(nations::table)
            .filter(nations::id.eq(&self.id))
            .set(self)
            .get_result(&mut conn)?;

        Ok(res)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable, SimpleObject, InputObject)]
#[diesel(table_name = nations)]
pub struct NewNation {
    pub creator_id: Uuid, // User
    pub nation_code: String,
    pub nation_name: String,
}

impl NewNation {
    pub fn new(creator_id: Uuid, nation_code: String, nation_name: String) -> Self {
        NewNation {
            creator_id,
            nation_code,
            nation_name,
        }
    }
}
