# DoD Metadata Model Refactoring

## Overview

This refactoring implements the **10 required baseline metadata fields** specified in pages 14-17 of the DoD Metadata Guidance document (January 2023). The implementation provides an elegant, type-safe, and extensible metadata structure compliant with DoD standards for NATO security classification conversions.

## DoD Metadata Guidance Summary (Pages 14-17)

The DoD Metadata Guidance establishes **minimum essential metadata requirements** organized into two categories:

### 1. Resource Description (7 fields)
- **Identifier**: Universal unique reference (GUID)
- **Authorization Reference**: Documented legal basis for mission activities
- **Originator**: Entity primarily responsible for generating the resource
- **Custodian**: Legally responsible organizational element
- **DataItemCreateDateTime**: Resource creation timestamp (UTC)
- **Description**: Brief overview of contents (abstract, summary)
- **Format**: File format, physical medium, or dimensions

### 2. Safeguarding & Sharing (3 fields)
- **Security Classification**: Highest classification level in the resource
- **Disclosure & Releasability**: Approved recipients (countries, organizations)
- **Handling Restrictions**: Non-classification limitations (CUI, PII, FOUO)

## Implementation Design

### Architecture Decisions

1. **Structured Types with JSON Storage**
   - Complex metadata fields (Originator, Custodian, etc.) use dedicated Rust structs
   - Stored as JSONB in PostgreSQL for flexibility and performance
   - Type-safe deserialization via `serde_json`

2. **Builder Pattern**
   - Fluent API for constructing metadata records
   - Optional fields can be added incrementally
   - Example:
     ```rust
     let metadata = NewMetadata::new(data_object_id, domain, description, timestamp)
         .with_originator(originator)
         .with_security_classification(classification)
         .with_tags(tags);
     ```

3. **GraphQL Integration**
   - Automatic parsing of JSON fields via resolver methods
   - Type-safe queries and mutations
   - Clean separation between database representation and API surface

4. **Backward Compatibility**
   - Legacy `domain` and `tags` fields preserved
   - Existing data continues to work without migration
   - Gradual adoption of new fields possible

## Data Structures

### Authorization Reference
```rust
pub struct AuthorizationReference {
    pub document_title: String,           // e.g., "Exercise Brave Squirrel 2025 Order"
    pub document_date: String,             // ISO 8601 format
    pub organization_name: Option<String>, // e.g., "Joint Chiefs of Staff"
    pub organization_contact: Option<String>,
}
```

### Originator
```rust
pub struct Originator {
    pub organization_name: String,          // Required
    pub organization_address: Option<String>,
    pub organization_email: Option<String>,
    pub organization_phone: Option<String>,
    pub poc_name: Option<String>,           // Point of Contact
    pub poc_role: Option<String>,
}
```

### Custodian
```rust
pub struct Custodian {
    pub organization_name: String,
    pub organization_address: Option<String>,
    pub organization_email: Option<String>,
    pub organization_phone: Option<String>,
}
```

### Format
```rust
pub struct Format {
    pub format_type: String,        // e.g., "Microsoft Word", "JPEG", "XML"
    pub size_bytes: Option<i64>,
    pub media_format: Option<String>,
}
```

### Security Classification
```rust
pub struct SecurityClassification {
    pub classification_level: String,         // e.g., "UNCLASSIFIED", "SECRET"
    pub reference_document_title: Option<String>,
    pub reference_document_date: Option<String>,
    pub classifier_organization: Option<String>,
    pub declassify_on: Option<NaiveDateTime>,
    pub retention_date: Option<NaiveDateTime>,
}
```

### Disclosure & Releasability
```rust
pub struct DisclosureReleasability {
    pub disclosure_category: Option<String>,        // e.g., "Category C"
    pub releasable_to: Vec<String>,                 // e.g., ["FVEY", "NATO"]
    pub recognized_organizations: Option<Vec<String>>,
}
```

### Handling Restrictions
```rust
pub struct HandlingRestrictions {
    pub handling_type: String,                  // e.g., "CUI", "FOUO", "PII"
    pub restrictions_description: Option<String>,
    pub authority: Option<String>,              // Policy or legislation reference
}
```

## Database Schema

The metadata table now includes all 10 required fields:

```sql
CREATE TABLE metadata (
    id UUID PRIMARY KEY,
    data_object_id UUID NOT NULL,

    -- Resource Description
    authorization_reference JSONB,
    originator JSONB,
    custodian JSONB,
    data_item_create_datetime TIMESTAMP NOT NULL,
    description TEXT NOT NULL,
    format JSONB,

    -- Safeguarding & Sharing
    security_classification JSONB,
    disclosure_releasability JSONB,
    handling_restrictions JSONB,

    -- Legacy fields
    domain VARCHAR(256) NOT NULL,
    tags TEXT[],

    -- Audit
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);
```

## Usage Examples

### Creating Compliant Metadata

```rust
use chrono::Utc;

// Create originator
let originator = Originator {
    organization_name: "Joint Staff J7".to_string(),
    organization_address: Some("116 Lakeview Pkwy, Suffolk, VA".to_string()),
    organization_email: Some("contact@example.mil".to_string()),
    organization_phone: Some("757-555-5555".to_string()),
    poc_name: Some("Maj Gen John Doe".to_string()),
    poc_role: Some("Lead Event Architect".to_string()),
};

// Create security classification
let classification = SecurityClassification {
    classification_level: "UNCLASSIFIED".to_string(),
    reference_document_title: Some("Chairman's Instruction XYZ".to_string()),
    reference_document_date: Some("2023-03-05".to_string()),
    classifier_organization: Some("Joint Chiefs of Staff".to_string()),
    declassify_on: None,
    retention_date: None,
};

// Create disclosure settings
let disclosure = DisclosureReleasability {
    disclosure_category: Some("Public".to_string()),
    releasable_to: vec!["Public".to_string()],
    recognized_organizations: None,
};

// Build metadata using fluent API
let metadata = NewMetadata::new(
    data_object_id,
    "training".to_string(),
    "Results of Exercise Brave Squirrel 2025".to_string(),
    Utc::now().naive_utc()
)
.with_originator(originator)
.with_security_classification(classification)
.with_disclosure_releasability(disclosure)
.with_handling_restrictions(HandlingRestrictions {
    handling_type: "None".to_string(),
    restrictions_description: None,
    authority: None,
});

// Save to database
Metadata::create(&metadata)?;
```

### Querying Metadata via GraphQL

```graphql
query GetMetadata($id: UUID!) {
    metadata(id: $id) {
        id
        description
        dataItemCreateDateTime

        originatorParsed {
            organizationName
            pocName
            pocRole
        }

        securityClassificationParsed {
            classificationLevel
            referenceDocumentTitle
        }

        disclosureReleasabilityParsed {
            releasableTo
        }

        handlingRestrictionsParsed {
            handlingType
        }
    }
}
```

## Benefits of This Design

### 1. **DoD Compliance**
- Implements all 10 required baseline metadata fields
- Follows guidance from pages 14-17 exactly
- Supports full audit trail and records management

### 2. **Type Safety**
- Compile-time checking of metadata structure
- IDE autocomplete for all fields
- Prevents invalid data at the application layer

### 3. **Flexibility**
- JSONB storage allows schema evolution
- New fields can be added without breaking changes
- Supports mission-specific extensions

### 4. **Performance**
- JSONB indexing for fast queries
- Efficient storage of complex nested structures
- GraphQL resolver caching

### 5. **Interoperability**
- Maps cleanly to NATO Core Metadata Specification (STANAG 5636)
- Compatible with Federal metadata standards (PM-ISE PO#3)
- Supports NARA records management requirements

## Alignment with DoD Guidance

This implementation directly addresses the key objectives from pages 14-17:

| Objective | Implementation |
|-----------|----------------|
| **Minimum baseline metadata** | All 10 required fields implemented |
| **Applied between creation and storage** | Timestamps and builder pattern support this |
| **Resource Description** | 7 dedicated fields with structured types |
| **Safeguarding & Sharing** | 3 dedicated fields for classification/handling |
| **Backward compatibility** | Legacy `domain` and `tags` preserved |
| **Extensibility** | JSONB allows community-specific additions |
| **Machine-readable** | JSON encoding for automated processing |
| **Human-readable** | GraphQL resolvers provide clean API |

## Migration Guide

### For Existing Data

1. **No immediate action required** - Legacy fields still work
2. **Gradual enrichment** - Add new fields as data is updated
3. **Batch updates** - Script available for bulk metadata enhancement

### For New Development

1. Use `NewMetadata::new()` with all required fields
2. Add optional fields via builder methods
3. Always populate at minimum:
   - `description`
   - `data_item_create_datetime`
   - `domain` (for legacy compatibility)

## Future Enhancements

- [ ] Validation helpers for classification levels
- [ ] Auto-population from user context
- [ ] Metadata templates for common scenarios
- [ ] Integration with external metadata catalogs
- [ ] Versioning and provenance tracking
- [ ] AI-assisted metadata generation

## References

- DoD Metadata Guidance, Version 1.0 (January 2023), pages 14-17
- NATO Core Metadata Specification (STANAG 5636)
- PM-ISE Priority Objective 3 (PO#3)
- NARA Metadata Requirements (36 CFR 1236.12)

## Questions or Issues?

Contact the development team or refer to the DoD Metadata Guidance document for authoritative guidance on metadata requirements.
