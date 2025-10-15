
use std::collections::HashMap;

use diesel::{prelude::*, r2d2::Error};


use crate::models::{ClassificationSchema, ConversionRequest, ConversionResponse};

/// Mapper that accepts a ConversionRequest, checks the incoming classification
/// against target NATO nation classifications and generates a ConversionResponse
pub struct Mapper {
    pub conversion_request: ConversionRequest,
    pub source_nation_schema: ClassificationSchema,
    pub target_nation_schemas: Vec<ClassificationSchema>,
}

impl Mapper {
    pub fn new(request: ConversionRequest) -> Self {

        let source_schema = ClassificationSchema::get_latest_by_nation_code(
            &request.source_nation_code)
            .expect("Unable to retrieve source schema");

        let target_schemas = ClassificationSchema::get_by_nation_codes(
            &request.target_nation_codes)
            .expect("Unable to retrieve target schemas");

        Mapper {
            conversion_request: request,
            source_nation_schema: source_schema,
            target_nation_schemas: target_schemas,
        }
    }

    pub fn generate_response(&self) -> Result<ConversionResponse> {

        let mut conversion_mappings = HashMap::new();

        let nato_equivalent = self.source_nation_schema.

        // take request security classification and for each target country, retrieve the converted classification
        for schema in &self.target_nation_schemas {

        }
        // generate the ConversionResponse

        // update the ConversionRequest to completed

        Ok(())
    }
}