

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

// A mapper struct with methods to run the API functionality
// A Request to the API will trigger the creation of a mapper that finds the correct ClassificationSchema, 
// identifies and authenticates an authority and, if validated, ingests the DataObject and Metadata and returns a ConversionResponse
pub struct ConversionMapper {
    pub schema: ClassificationSchema,
}

pub struct ConversionResponse {
    pub id: Uuid,
    pub subject_data_id: Uuid, // SubjectData
    pub nato_equivalent: NatoClassification,
    pub target_nation_classification: Hashmap<NationCode, String>,
    pub created_at: NaiveDate,
    pub updated_at: NaiveDate,
    pub expires_at: Option<NaiveDate>,
}

pub struct DataAccess {
    pub id: Uuid,
    pub person_id: Uuid,
    pub approved_access_level: String,       // AccessLevel
    pub approved_access_granularity: String, // Granularity
}
