use std::fmt::Debug;

use async_graphql::*;
use chrono::prelude::*;
use diesel::dsl::count;
use diesel::prelude::*;
use diesel::{
    self, BoolExpressionMethods, ExpressionMethods, Insertable, PgTextExpressionMethods, Queryable,
};
use diesel::{QueryDsl, RunQueryDsl};
use diesel_derive_enum::DbEnum;
use rand::{
    Rng,
    distributions::{Distribution, Standard},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::connection;

use crate::{database, schema::*};

use crate::models::{Authority, Nation};

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
#[diesel(belongs_to(Person))]
#[diesel(belongs_to(Skill))]
#[diesel(belongs_to(Organization))]
#[diesel(table_name = capabilities)]
#[graphql(complex)]
/// A request for security classification conversion to the middleware
pub struct Request {
    pub id: Uuid,
    pub creator_id: Uuid // User
    pub data_object_id: Uuid // DataObject
    pub source_nation_code: NationCode,
    pub target_nation_codes: Vec<String>,
    pub national_classification: String,
    //pub context_group: Option<String>, // Used for sending only to certain groups, missions or lists.
    pub created_at: NaiveDate,
    pub updated_at: NaiveDate,
    pub completed_at: NaiveDate,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    DbEnum,
    Serialize,
    Deserialize,
    Enum,
    PartialOrd,
    Ord,
    Display,
)]

// Graphql
#[ComplexObject]
impl Request {
    pub async fn person(&self) -> Result<Person> {
        Person::get_by_id(&self.person_id)
    }

    pub async fn skill_name(&self) -> Result<String> {
        Skill::get_name_by_id(&self.skill_id)
    }

    pub async fn skill(&self) -> Result<Skill> {
        Skill::get_by_id(&self.skill_id)
    }

    /// Detailed view of validations for this request
    pub async fn validations(&self) -> Result<Vec<Validation>> {
        Validation::get_by_request_id(&self.id)
    }
}

// Non Graphql
impl Request {
    pub fn create(request: &NewRequest) -> Result<Request> {
        let mut conn = connection()?;

        let res = diesel::insert_into(capabilities::table)
            .values(request)
            .get_result(&mut conn)?;

        Ok(res)
    }

    pub fn batch_create(capabilities: &Vec<NewRequest>) -> Result<usize> {
        let mut conn = connection()?;

        let res = diesel::insert_into(capabilities::table)
            .values(capabilities)
            .execute(&mut conn)?;

        Ok(res)
    }

    pub fn get_or_create(request: &NewRequest) -> Result<Request> {
        let mut conn = connection()?;

        let res = capabilities::table
            .filter(
                capabilities::person_id
                    .eq(&request.person_id)
                    .and(capabilities::skill_id.eq(&request.skill_id)),
            )
            .distinct()
            .first(&mut conn);

        let request = match res {
            Ok(p) => p,
            Err(e) => {
                // Request not found
                println!("{:?}", e);
                let p = Request::create(request).expect("Unable to create request");
                p
            }
        };
        Ok(request)
    }

    pub fn get_all() -> Result<Vec<Self>> {
        let mut conn = database::connection()?;
        let res = capabilities::table.load::<Request>(&mut conn)?;
        Ok(res)
    }

    pub fn get_count(count: i64) -> Result<Vec<Self>> {
        let mut conn = database::connection()?;
        let res = capabilities::table
            .limit(count)
            .load::<Request>(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_id(id: &Uuid) -> Result<Self> {
        let mut conn = database::connection()?;
        let res = capabilities::table
            .filter(capabilities::id.eq(id))
            .first(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_skill_id(id: Uuid) -> Result<Vec<Self>> {
        let mut conn = connection()?;

        let res = capabilities::table
            .filter(capabilities::skill_id.eq(id))
            .load::<Request>(&mut conn)?;

        Ok(res)
    }

    pub fn get_by_skill_id_and_level(id: Uuid, level: RequestLevel) -> Result<Vec<Self>> {
        let mut conn = connection()?;

        let res = capabilities::table
            .filter(capabilities::skill_id.eq(id))
            .filter(capabilities::validated_level.ge(level))
            .load::<Request>(&mut conn)?;

        Ok(res)
    }

    pub fn get_by_name(name: &String) -> Result<Vec<Self>> {
        let mut conn = connection()?;

        let res = capabilities::table
            .filter(
                capabilities::name_en
                    .ilike(format!("%{}%", name))
                    .or(capabilities::name_fr.ilike(format!("%{}%", name))),
            )
            .load::<Request>(&mut conn)?;

        Ok(res)
    }

    pub fn get_by_name_and_level(name: &String, level: RequestLevel) -> Result<Vec<Self>> {
        let mut conn = connection()?;

        let res = capabilities::table
            .filter(
                capabilities::name_en
                    .ilike(format!("%{}%", name))
                    .or(capabilities::name_fr.ilike(format!("%{}%", name))),
            )
            .filter(capabilities::self_identified_level.eq(level))
            .load::<Request>(&mut conn)?;

        Ok(res)
    }

    pub fn get_by_domain_and_level(
        domain: &SkillDomain,
        level: RequestLevel,
    ) -> Result<Vec<Self>> {
        let mut conn = connection()?;

        let res = capabilities::table
            .filter(capabilities::domain.eq(domain))
            .filter(capabilities::self_identified_level.eq(level))
            .load::<Request>(&mut conn)?;

        Ok(res)
    }

    pub fn get_by_person_id(id: Uuid) -> Result<Vec<Self>> {
        let mut conn = connection()?;

        let res = capabilities::table
            .filter(capabilities::person_id.eq(id))
            .load::<Request>(&mut conn)?;

        Ok(res)
    }

    pub fn get_level_counts_by_name(name: String) -> Result<Vec<RequestCount>> {
        let mut conn = connection()?;

        let skill_id = Skill::get_top_skill_id_by_name(name)?;

        let res: Vec<(String, SkillDomain, Option<RequestLevel>, i64)> = capabilities::table
            .filter(capabilities::skill_id.eq(skill_id))
            .group_by((
                capabilities::domain,
                capabilities::validated_level,
                capabilities::name_en,
            ))
            .select((
                capabilities::name_en,
                capabilities::domain,
                capabilities::validated_level,
                count(capabilities::id),
            ))
            .order_by((capabilities::name_en, capabilities::validated_level))
            .load::<(String, SkillDomain, Option<RequestLevel>, i64)>(&mut conn)?;

        // Convert res into RequestCountStruct
        let mut counts: Vec<RequestCount> = Vec::new();

        for r in res {
            let count = RequestCount::from(r);
            counts.push(count);
        }

        Ok(counts)
    }

    pub fn get_level_counts_by_domain(domain: SkillDomain) -> Result<Vec<RequestCount>> {
        let mut conn = connection()?;

        let res: Vec<(String, SkillDomain, Option<RequestLevel>, i64)> = capabilities::table
            .filter(capabilities::domain.eq(domain))
            .group_by((
                capabilities::domain,
                capabilities::validated_level,
                capabilities::name_en,
            ))
            .select((
                capabilities::name_en,
                capabilities::domain,
                capabilities::validated_level,
                count(capabilities::id),
            ))
            .order_by((capabilities::name_en, capabilities::validated_level))
            .load::<(String, SkillDomain, Option<RequestLevel>, i64)>(&mut conn)?;

        // Convert res into RequestCountStruct
        let mut counts: Vec<RequestCount> = Vec::new();

        for r in res {
            let count = RequestCount::from(r);
            counts.push(count);
        }

        Ok(counts)
    }

    /// Updates a Request based on a new validation
    pub fn update_from_validation(&mut self, validated_level: &RequestLevel) -> Result<Self> {
        self.validation_values
            .push(Some(ValidatedLevel::get_value_from_request_level(
                validated_level,
            )));

        let values: Option<Vec<i64>> = self.validation_values.clone().into_iter().collect();

        let values = values.unwrap();

        let validation_average: i64 = values.iter().sum::<i64>() / values.len() as i64;

        let validated_level = ValidatedLevel::get_request_level_from_value(&validation_average);

        self.validated_level = Some(validated_level);

        self.update()
    }

    /// Updates a Request based on a vector of validations
    pub fn update_from_batch_validations(
        &mut self,
        validated_levels: &Vec<RequestLevel>,
    ) -> Result<Self> {
        for v in validated_levels {
            self.validation_values
                .push(Some(ValidatedLevel::get_value_from_request_level(v)));
        }

        let values: Option<Vec<i64>> = self.validation_values.clone().into_iter().collect();

        let values = values.unwrap();

        let validation_average: i64 = values.iter().sum::<i64>() / values.len() as i64;

        let validated_level = ValidatedLevel::get_request_level_from_value(&validation_average);

        self.validated_level = Some(validated_level);

        self.update()
    }

    /// Updates a Request based on changed data
    pub fn update(&self) -> Result<Self> {
        let mut conn = database::connection()?;

        let res = diesel::update(capabilities::table)
            .filter(capabilities::id.eq(&self.id))
            .set(self)
            .get_result(&mut conn)?;

        Ok(res)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable, InputObject)]
#[diesel(table_name = capabilities)]
pub struct NewRequest {
    pub name_en: String,
    pub name_fr: String,
    pub domain: SkillDomain,
    pub person_id: Uuid, // Person
    pub skill_id: Uuid,  // Skill
    pub organization_id: Uuid,
    pub self_identified_level: RequestLevel,
    pub validation_values: Vec<i64>,
}

impl NewRequest {
    pub fn new(
        person_id: Uuid,       // Person
        skill_id: Uuid,        // Skill
        organization_id: Uuid, // Organization
        self_identified_level: RequestLevel,
    ) -> Self {
        let skill = Skill::get_by_id(&skill_id).expect("Unable to get skill");

        let self_identified_value: i64 =
            ValidatedLevel::get_value_from_request_level(&self_identified_level);

        NewRequest {
            name_en: skill.name_en,
            name_fr: skill.name_fr,
            domain: skill.domain,
            person_id: person_id,
            skill_id: skill.id,
            organization_id: organization_id,
            self_identified_level,
            validation_values: vec![self_identified_value],
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, SimpleObject)]
pub struct RequestCount {
    pub name: String,
    pub domain: SkillDomain,
    pub level: String,
    pub counts: i64,
}

impl From<(String, SkillDomain, Option<RequestLevel>, i64)> for RequestCount {
    fn from(
        (name, domain, level, counts): (String, SkillDomain, Option<RequestLevel>, i64),
    ) -> Self {
        RequestCount {
            name,
            domain,
            level: level
                .expect("Unable to translate Validated Level")
                .to_string(),
            counts,
        }
    }
}

impl RequestCount {
    pub fn new(name: String, domain: SkillDomain, level: String, counts: i64) -> Self {
        RequestCount {
            name,
            domain,
            level,
            counts,
        }
    }
}
