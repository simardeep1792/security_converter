pub struct ClassificationRequest {
    pub id: Uuid,
    pub creator_id: Uuid,     // User
    pub data_object_id: Uuid, // DataObject
    pub source_nation_code: NationCode,
    pub target_nation_codes: Vec<NationCode>,
    pub national_classification: String,
    //pub context_group: Option<String>, // Used for sending only to certain groups, missions or lists.
    pub created_at: NaiveDate,
    pub updated_at: NaiveDate,
    pub completed_at: NaiveDate,
}

/// Struct representing the data being shared or accessed
pub struct DataObject {
    pub id: Uuid,
    pub creator_id: Uuid, // User
    pub title: String,
    pub description: String,
    pub created_at: NaiveDate,
    pub updated_at: NaiveDate,
}

// Model for sharing to a mission or coalition group - this can come later
pub struct ContextGroup {
    pub id: Uuid,
    pub creator_id: Uuid, // User
    pub name: String,
    pub description: String,
    pub nations: Vec<NationCode>,
    pub max_classification: NatoClassification,
    pub created_at: NaiveDate,
    pub updated_at: NaiveDate,
    pub expires_at: Option<NaiveDate>,
}

pub struct ClassificationSchema {
    pub id: Uuid,
    pub creator_id: Uuid, // User
    pub nation_code: NationCode,
    // Conversions to NATO
    pub to_nato_unclassified: String,
    pub to_nato_restricted: String,
    pub to_nato_confidential: String,
    pub to_nato_secret: String,
    pub to_nato_top_secret: String,
    // Conversions from NATO
    pub from_nato_unclassified: String,
    pub from_nato_restricted: String,
    pub from_nato_confidential: String,
    pub from_nato_secret: String,
    pub from_nato_top_secret: String,
    // Other details
    pub caveats: String,
    //pub trust_matrix: Hashmap<NationCode, TrustLevel>,
    pub version: String,
    pub authority_id: Uuid, // Authority
    pub created_at: NaiveDate,
    pub updated_at: NaiveDate,
    pub expires_at: Option<NaiveDate>,
}

pub struct ClassificationMapper {
    pub schema: ClassificationSchema,
}

pub struct Authority {
    pub id: Uuid,
    pub creator_id: Uuid, // User
    pub nation_id: Uuid,  // Nation
    pub name: String,
    pub email: String,
    pub phone: String,
    pub created_at: NaiveDate,
    pub updated_at: NaiveDate,
    pub expires_at: Option<NaiveDate>,
}

pub struct ClassificationResponse {
    pub id: Uuid,
    pub subject_data_id: Uuid, // SubjectData
    pub nato_equivalent: NatoClassification,
    pub target_nation_classification: Hashmap<NationCode, String>,
    pub created_at: NaiveDate,
    pub updated_at: NaiveDate,
    pub expires_at: Option<NaiveDate>,
}

/// Metadata required for ACP240 and other interop standards
pub struct MetaData {
    pub id: Uuid,
    pub subject_data_id: Uuid,
    pub domain: String,
    pub tags: Vec<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub struct Nation {
    pub id: Uuid,
    pub creator_id: Uuid,    // User
    pub nation_code: String, // 3 letter nation code
    pub nation_name: String,
    pub created_at: NaiveDate,
    pub updated_at: NaiveDate,
}

pub enum NatoClassification {
    Unclassified,
    Restricted,
    Confidential,
    Secret,
    TopSecret,
}

pub enum TrustLevel {
    Low,
    Medium,
    High,
}

pub struct DataAccess {
    pub id: Uuid,
    pub person_id: Uuid,
    pub approved_access_level: String,       // AccessLevel
    pub approved_access_granularity: String, // Granularity
}
