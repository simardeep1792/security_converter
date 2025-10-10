use async_graphql::{Data, Error};
use diesel::{self, RunQueryDsl};
use rand::Rng;
use rand::seq::SliceRandom;
use uuid::Uuid;

use crate::models::{
    Authority, ConversionRequest, DataObject, InsertableConversionRequest, InsertableDataObject,
    InsertableMetadata, Nation, NewAuthority, NewClassificationSchema, NewDataObject, NewMetadata,
    NewNation, User,
};
use crate::progress::progress::ProgressLogger;
use crate::{database, schema::*};

/// Creates basic test objects in the database for testing
pub fn pre_populate_db_schema() -> Result<(), Error> {
    // Get all users to assign as creators
    let users = User::get_all()?;
    if users.is_empty() {
        return Err(Error::new(
            "No users found in database. Please create users first.",
        ));
    }

    let mut rng = rand::thread_rng();

    // ========== Create NATO Nations ==========
    println!("\n========== Creating NATO Nations ==========");
    let nations_data = vec![
        ("USA", "United States"),
        ("GBR", "United Kingdom"),
        ("FRA", "France"),
        ("DEU", "Germany"),
        ("ITA", "Italy"),
        ("CAN", "Canada"),
        ("ESP", "Spain"),
        ("NLD", "Netherlands"),
        ("POL", "Poland"),
        ("TUR", "Turkey"),
        ("NOR", "Norway"),
        ("BEL", "Belgium"),
        ("DNK", "Denmark"),
        ("PRT", "Portugal"),
        ("CZE", "Czech Republic"),
        ("HUN", "Hungary"),
        ("GRC", "Greece"),
        ("ROU", "Romania"),
        ("BGR", "Bulgaria"),
        ("SVK", "Slovakia"),
    ];

    let mut new_nations = Vec::new();
    for (code, name) in nations_data.iter() {
        let creator = users.choose(&mut rng).unwrap();
        new_nations.push(NewNation::new(
            creator.id,
            code.to_string(),
            name.to_string(),
        ));
    }

    let mut progress_nations =
        ProgressLogger::new("Inserting Nations".to_owned(), new_nations.len());

    // Batch insert nations
    let mut conn = database::connection()?;
    let inserted_count = diesel::insert_into(nations::table)
        .values(&new_nations)
        .execute(&mut conn)?;

    progress_nations.done();
    println!("✓ Inserted {} nations", inserted_count);

    // Retrieve created nations for foreign key references
    let created_nations = Nation::get_all()?;

    // ========== Create Authorities ==========
    println!("\n========== Creating Authorities ==========");
    let authority_titles = vec![
        "National Security Authority",
        "Defense Intelligence Agency",
        "Ministry of Defence Security",
        "National Classification Office",
        "Military Intelligence Directorate",
        "Security Clearance Division",
    ];

    let mut new_authorities = Vec::new();

    for nation in created_nations.iter() {
        let num_authorities = rng.gen_range(1..=3);

        for i in 0..num_authorities {
            let creator = users.choose(&mut rng).unwrap();
            let title = authority_titles.choose(&mut rng).unwrap();

            let name = format!("{} - {}", nation.nation_name, title);
            let email = format!(
                "{}{}@{}.nato.int",
                title.to_lowercase().replace(" ", "."),
                if i > 0 {
                    format!("{}", i)
                } else {
                    String::new()
                },
                nation.nation_code.to_lowercase()
            );
            let phone = format!(
                "+{}-{}-{}-{}",
                rng.gen_range(1..999),
                rng.gen_range(100..999),
                rng.gen_range(100..999),
                rng.gen_range(1000..9999)
            );

            // 30% chance to have expiration date
            let expires_at = if rng.gen_bool(0.3) {
                Some(
                    chrono::Utc::now().naive_utc()
                        + chrono::Duration::days(rng.gen_range(365..1095)),
                )
            } else {
                None
            };

            new_authorities.push(NewAuthority::new(
                creator.id, nation.id, name, email, phone, expires_at,
            ));
        }
    }

    let mut progress_authorities =
        ProgressLogger::new("Inserting Authorities".to_owned(), new_authorities.len());

    // Batch insert authorities
    let inserted_authorities = diesel::insert_into(authorities::table)
        .values(&new_authorities)
        .execute(&mut conn)?;

    progress_authorities.done();
    println!("✓ Inserted {} authorities", inserted_authorities);

    // Retrieve created authorities for foreign key references
    let created_authorities = Authority::get_all()?;

    // ========== Create Classification Schemas ==========
    println!("\n========== Creating Classification Schemas ==========");

    // Realistic classification mappings for various NATO nations
    let classification_mappings = vec![
        (
            "USA",
            vec![
                (
                    "UNCLASSIFIED",
                    "CONFIDENTIAL",
                    "CONFIDENTIAL",
                    "SECRET",
                    "TOP SECRET",
                ),
                (
                    "UNCLASSIFIED",
                    "CONFIDENTIAL",
                    "CONFIDENTIAL",
                    "SECRET",
                    "TOP SECRET",
                ),
            ],
        ),
        (
            "GBR",
            vec![
                (
                    "OFFICIAL",
                    "OFFICIAL-SENSITIVE",
                    "SECRET",
                    "SECRET",
                    "TOP SECRET",
                ),
                (
                    "OFFICIAL",
                    "OFFICIAL-SENSITIVE",
                    "SECRET",
                    "SECRET",
                    "TOP SECRET",
                ),
            ],
        ),
        (
            "FRA",
            vec![
                (
                    "Non Protégé",
                    "Diffusion Restreinte",
                    "Confidentiel Défense",
                    "Secret Défense",
                    "Très Secret Défense",
                ),
                (
                    "Non Protégé",
                    "Diffusion Restreinte",
                    "Confidentiel Défense",
                    "Secret Défense",
                    "Très Secret Défense",
                ),
            ],
        ),
        (
            "DEU",
            vec![
                (
                    "Offen",
                    "VS-NfD",
                    "VS-Vertraulich",
                    "Geheim",
                    "Streng Geheim",
                ),
                (
                    "Offen",
                    "VS-NfD",
                    "VS-Vertraulich",
                    "Geheim",
                    "Streng Geheim",
                ),
            ],
        ),
        (
            "ITA",
            vec![
                (
                    "Non Classificato",
                    "Riservato",
                    "Riservatissimo",
                    "Segreto",
                    "Segretissimo",
                ),
                (
                    "Non Classificato",
                    "Riservato",
                    "Riservatissimo",
                    "Segreto",
                    "Segretissimo",
                ),
            ],
        ),
        (
            "CAN",
            vec![
                (
                    "UNCLASSIFIED",
                    "PROTECTED A",
                    "PROTECTED B",
                    "SECRET",
                    "TOP SECRET",
                ),
                (
                    "UNCLASSIFIED",
                    "PROTECTED A",
                    "PROTECTED B",
                    "SECRET",
                    "TOP SECRET",
                ),
            ],
        ),
        (
            "ESP",
            vec![
                (
                    "No Clasificado",
                    "Difusión Limitada",
                    "Confidencial",
                    "Secreto",
                    "Alto Secreto",
                ),
                (
                    "No Clasificado",
                    "Difusión Limitada",
                    "Confidencial",
                    "Secreto",
                    "Alto Secreto",
                ),
            ],
        ),
        (
            "NLD",
            vec![
                (
                    "Niet-Gerubriceerd",
                    "Departementaal Vertrouwelijk",
                    "Confidentieel",
                    "Geheim",
                    "Zeer Geheim",
                ),
                (
                    "Niet-Gerubriceerd",
                    "Departementaal Vertrouwelijk",
                    "Confidentieel",
                    "Geheim",
                    "Zeer Geheim",
                ),
            ],
        ),
        (
            "POL",
            vec![
                ("Jawne", "Zastrzeżone", "Poufne", "Tajne", "Ściśle Tajne"),
                ("Jawne", "Zastrzeżone", "Poufne", "Tajne", "Ściśle Tajne"),
            ],
        ),
        (
            "TUR",
            vec![
                ("Açık", "Hizmete Özel", "Özel", "Gizli", "Çok Gizli"),
                ("Açık", "Hizmete Özel", "Özel", "Gizli", "Çok Gizli"),
            ],
        ),
    ];

    let caveats_options = vec![
        "NOFORN",
        "REL TO NATO",
        "EYES ONLY",
        "ORIGINATOR CONTROLLED",
        "RELEASABLE TO",
        "NATO UNCLASSIFIED",
        "",
    ];

    let mut new_classification_schemas = Vec::new();

    for nation in created_nations.iter() {
        // Find matching classification mapping
        if let Some(mapping) = classification_mappings
            .iter()
            .find(|(code, _)| code == &nation.nation_code.as_str())
        {
            // Get authorities for this nation
            let nation_authorities: Vec<&Authority> = created_authorities
                .iter()
                .filter(|a| a.nation_id == nation.id)
                .collect();

            if nation_authorities.is_empty() {
                continue;
            }

            // Create 1-2 versions for this nation
            let num_versions = if rng.gen_bool(0.3) { 2 } else { 1 };

            for version_num in 1..=num_versions {
                let creator = users.choose(&mut rng).unwrap();
                let authority = nation_authorities.choose(&mut rng).unwrap();
                let caveat = caveats_options.choose(&mut rng).unwrap();

                // Use the appropriate mapping tuple (all same for realistic data)
                let levels = mapping.1[0];

                // 20% chance to have expiration date
                let expires_at = if rng.gen_bool(0.2) {
                    Some(
                        chrono::Utc::now().naive_utc()
                            + chrono::Duration::days(rng.gen_range(730..1825)), // 2-5 years
                    )
                } else {
                    None
                };

                new_classification_schemas.push(NewClassificationSchema::new(
                    creator.id,
                    nation.nation_code.clone(),
                    // TO NATO mappings
                    levels.0.to_string(),
                    levels.1.to_string(),
                    levels.2.to_string(),
                    levels.3.to_string(),
                    levels.4.to_string(),
                    // FROM NATO mappings (same as TO for simplicity)
                    levels.0.to_string(),
                    levels.1.to_string(),
                    levels.2.to_string(),
                    levels.3.to_string(),
                    levels.4.to_string(),
                    caveat.to_string(),
                    format!("v{}.0", version_num),
                    authority.id,
                    expires_at,
                ));
            }
        } else {
            // For nations without specific mappings, create a generic one
            let creator = users.choose(&mut rng).unwrap();
            let nation_authorities: Vec<&Authority> = created_authorities
                .iter()
                .filter(|a| a.nation_id == nation.id)
                .collect();

            if let Some(authority) = nation_authorities.choose(&mut rng) {
                let caveat = caveats_options.choose(&mut rng).unwrap();

                let expires_at = if rng.gen_bool(0.2) {
                    Some(
                        chrono::Utc::now().naive_utc()
                            + chrono::Duration::days(rng.gen_range(730..1825)),
                    )
                } else {
                    None
                };

                new_classification_schemas.push(NewClassificationSchema::new(
                    creator.id,
                    nation.nation_code.clone(),
                    "UNCLASSIFIED".to_string(),
                    "RESTRICTED".to_string(),
                    "CONFIDENTIAL".to_string(),
                    "SECRET".to_string(),
                    "TOP SECRET".to_string(),
                    "UNCLASSIFIED".to_string(),
                    "RESTRICTED".to_string(),
                    "CONFIDENTIAL".to_string(),
                    "SECRET".to_string(),
                    "TOP SECRET".to_string(),
                    caveat.to_string(),
                    "v1.0".to_string(),
                    authority.id,
                    expires_at,
                ));
            }
        }
    }

    let mut progress_schemas = ProgressLogger::new(
        "Inserting Classification Schemas".to_owned(),
        new_classification_schemas.len(),
    );

    // Batch insert classification schemas
    let inserted_schemas = diesel::insert_into(classification_schemas::table)
        .values(&new_classification_schemas)
        .execute(&mut conn)?;

    progress_schemas.done();
    println!("✓ Inserted {} classification schemas", inserted_schemas);

    // ========== Create Metadata ==========
    println!("\n========== Creating Metadata ==========");
    let metadata_domains = vec![
        (
            "INTEL",
            vec![
                "sigint",
                "humint",
                "geoint",
                "osint",
                "classified",
                "top-secret",
            ],
        ),
        (
            "CYBER",
            vec![
                "cyber-defense",
                "network-security",
                "encryption",
                "firewall",
                "threat-intelligence",
            ],
        ),
        (
            "OPERATIONS",
            vec![
                "tactical",
                "strategic",
                "mission-critical",
                "operational-security",
                "field-ops",
            ],
        ),
        (
            "LOGISTICS",
            vec![
                "supply-chain",
                "transportation",
                "equipment",
                "maintenance",
                "procurement",
            ],
        ),
        (
            "COMMUNICATIONS",
            vec!["comms", "radio", "satellite", "secure-voice", "data-link"],
        ),
        (
            "NUCLEAR",
            vec![
                "nuclear-security",
                "wmd",
                "nonproliferation",
                "arms-control",
                "strategic-weapons",
            ],
        ),
        (
            "COUNTERTERRORISM",
            vec![
                "ct-ops",
                "threat-assessment",
                "surveillance",
                "counterintelligence",
                "security-clearance",
            ],
        ),
        (
            "MARITIME",
            vec![
                "naval-ops",
                "submarine",
                "surface-warfare",
                "maritime-patrol",
                "anti-submarine",
            ],
        ),
        (
            "AEROSPACE",
            vec![
                "air-defense",
                "fighter-ops",
                "reconnaissance",
                "space-operations",
                "satellite-imagery",
            ],
        ),
        (
            "SPECIAL-OPS",
            vec![
                "special-forces",
                "covert-ops",
                "direct-action",
                "reconnaissance",
                "hostage-rescue",
            ],
        ),
    ];

    // ========== Create DataObjects ==========
    println!("\n========== Creating DataObjects ==========");
    let data_object_templates = vec![
        (
            "Operation Report",
            "Detailed operational report for recent military exercise",
        ),
        (
            "Intelligence Assessment",
            "Strategic intelligence assessment of regional threats",
        ),
        (
            "Classified Briefing",
            "Executive briefing on classified security matters",
        ),
        (
            "Mission Planning Document",
            "Comprehensive mission planning and coordination document",
        ),
        (
            "Security Protocol",
            "Standard operating procedures for classified information handling",
        ),
        (
            "Threat Analysis",
            "Analysis of emerging security threats and countermeasures",
        ),
        (
            "Equipment Specifications",
            "Technical specifications for classified military equipment",
        ),
        (
            "Communications Plan",
            "Secure communications plan for joint operations",
        ),
        (
            "Training Manual",
            "Classified training manual for specialized personnel",
        ),
        (
            "Strategic Assessment",
            "Long-term strategic assessment of defense capabilities",
        ),
        (
            "Incident Report",
            "After-action report for security incident",
        ),
        (
            "Intelligence Summary",
            "Weekly intelligence summary and threat briefing",
        ),
        (
            "Operational Order",
            "Classified operational order for coordinated mission",
        ),
        (
            "Capability Analysis",
            "Analysis of allied capabilities and interoperability",
        ),
        (
            "Security Clearance Review",
            "Review and assessment of security clearance procedures",
        ),
    ];

    let num_data_objects = 30;
    let mut new_data_objects = Vec::new();
    let mut new_metadata = Vec::new();

    for i in 0..num_data_objects {
        let creator = users.choose(&mut rng).unwrap();
        let template = data_object_templates.choose(&mut rng).unwrap();

        let title = if i % 3 == 0 {
            format!("{} - {}", template.0, chrono::Utc::now().format("%Y-%m-%d"))
        } else {
            format!("{} #{}", template.0, rng.gen_range(1000..9999))
        };

        let description = format!(
            "{}. Created for testing and validation purposes.",
            template.1
        );

        new_data_objects.push(NewDataObject::new(creator.id, title, description));
    }

    // Batch insert data objects
    let inserted_data_objects = diesel::insert_into(data_objects::table)
        .values(&new_data_objects)
        .execute(&mut conn)?;

    let data_object_ids = DataObject::get_all()?;

    for elem in data_object_ids.iter() {
        let domain = metadata_domains.choose(&mut rng).unwrap();
        let tags = &domain.1;
        new_metadata.push(NewMetadata::new(
            elem.id,
            domain.0.to_string(),
            tags.choose_multiple(&mut rng, 2)
                .map(|s| Some(s.to_string()))
                .collect(),
        ));
    }

    // Batch insert metadata
    let inserted_metadata = diesel::insert_into(metadata::table)
        .values(&new_metadata)
        .execute(&mut conn)?;

    let mut progress_data_objects =
        ProgressLogger::new("Inserting DataObjects".to_owned(), new_data_objects.len());

    progress_data_objects.done();
    println!("✓ Inserted {} data objects", inserted_data_objects);

    // ========== Create Conversion Requests ==========
    println!("\n========== Creating Conversion Requests ==========");

    let classification_levels = vec!["UNCLASSIFIED", "RESTRICTED", "CONFIDENTIAL", "SECRET", "TOP SECRET"];
    let num_conversion_requests = 30;
    let mut created_conversion_requests = 0;

    let conversion_titles = vec![
        "Intelligence Report",
        "Tactical Assessment",
        "Strategic Document",
        "Operational Brief",
        "Security Analysis",
        "Mission Data",
        "Threat Intelligence",
        "Classified Communication",
        "Defense Protocol",
        "Military Exercise Report",
    ];

    let conversion_domains = vec![
        "INTEL", "CYBER", "OPERATIONS", "LOGISTICS", "COMMUNICATIONS",
        "NUCLEAR", "COUNTERTERRORISM", "MARITIME", "AEROSPACE", "SPECIAL-OPS",
    ];

    for i in 0..num_conversion_requests {
        let creator = users.choose(&mut rng).unwrap();
        let authority = created_authorities.choose(&mut rng).unwrap();
        let source_nation = created_nations.choose(&mut rng).unwrap();

        // Select 1-3 random target nations (different from source)
        let num_targets = rng.gen_range(1..=3);
        let mut target_nations: Vec<String> = created_nations
            .iter()
            .filter(|n| n.nation_code != source_nation.nation_code)
            .collect::<Vec<_>>()
            .choose_multiple(&mut rng, num_targets)
            .map(|n| n.nation_code.clone())
            .collect();

        // Ensure at least one target nation
        if target_nations.is_empty() {
            target_nations.push(
                created_nations
                    .iter()
                    .find(|n| n.nation_code != source_nation.nation_code)
                    .unwrap_or(&source_nation)
                    .nation_code
                    .clone()
            );
        }

        // Create insertable data object for the conversion request
        let title_template = conversion_titles.choose(&mut rng).unwrap();
        let title = format!("{} - {} #{}", title_template, source_nation.nation_code, rng.gen_range(1000..9999));
        let description = format!(
            "Conversion request for {} from {} to {:?}. Created for testing purposes.",
            title_template.to_lowercase(),
            source_nation.nation_name,
            target_nations
        );

        let insertable_data_object = InsertableDataObject {
            title: title.clone(),
            description: description.clone(),
        };

        // Create insertable metadata
        let domain = conversion_domains.choose(&mut rng).unwrap();

        let default_tags = &vec!["general", "nato", "classified"];
        
        let tags = metadata_domains
            .iter()
            .find(|(d, _)| d == domain)
            .map(|(_, tags)| tags)
            .unwrap_or(default_tags);

        let num_tags = rng.gen_range(1..=3);

        let selected_tags: Vec<Option<String>> = tags
            .choose_multiple(&mut rng, num_tags)
            .map(|s| Some(s.to_string()))
            .collect();

        let insertable_metadata = InsertableMetadata {
            domain: domain.to_string(),
            tags: selected_tags,
        };

        // Create the conversion request payload
        let conversion_payload = InsertableConversionRequest {
            user_id: creator.id,
            authority_id: authority.id,
            data_object: insertable_data_object,
            metadata: insertable_metadata,
            source_nation_code: source_nation.nation_code.clone(),
            target_nation_codes: target_nations,
        };

        // Process the payload to create the conversion request
        match ConversionRequest::process_payload(&conversion_payload) {
            Ok(_) => {
                created_conversion_requests += 1;
            }
            Err(e) => {
                eprintln!("Failed to create conversion request {}: {:?}", i + 1, e);
            }
        }
    }

    let mut progress_conversion_requests =
        ProgressLogger::new("Inserting Conversion Requests".to_owned(), num_conversion_requests);
    progress_conversion_requests.done();
    println!("✓ Inserted {} conversion requests", created_conversion_requests);

    // ========== Summary ==========
    println!("\n========================================");
    println!("✓ Database population complete!");
    println!("========================================");
    println!("  • {} Nations", inserted_count);
    println!("  • {} Authorities", inserted_authorities);
    println!("  • {} Classification Schemas", inserted_schemas);
    println!("  • {} Metadata domains", inserted_metadata);
    println!("  • {} DataObjects", inserted_data_objects);
    println!("  • {} Conversion Requests", created_conversion_requests);
    println!("========================================\n");

    Ok(())
}

pub fn gen_rand_number() -> String {
    let mut rng = rand::thread_rng();

    let rand_num: String = (0..11)
        .map(|_| {
            let i = rng.gen_range(0..10);
            i.to_string()
        })
        .collect();

    rand_num
}
