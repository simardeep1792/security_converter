// @generated automatically by Diesel CLI.

diesel::table! {
    authorities (id) {
        id -> Uuid,
        creator_id -> Uuid,
        nation_id -> Uuid,
        #[max_length = 256]
        name -> Varchar,
        #[max_length = 128]
        email -> Varchar,
        #[max_length = 32]
        phone -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        expires_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    classification_schemas (id) {
        id -> Uuid,
        creator_id -> Uuid,
        #[max_length = 3]
        nation_code -> Varchar,
        #[max_length = 128]
        to_nato_unclassified -> Varchar,
        #[max_length = 128]
        to_nato_restricted -> Varchar,
        #[max_length = 128]
        to_nato_confidential -> Varchar,
        #[max_length = 128]
        to_nato_secret -> Varchar,
        #[max_length = 128]
        to_nato_top_secret -> Varchar,
        #[max_length = 128]
        from_nato_unclassified -> Varchar,
        #[max_length = 128]
        from_nato_restricted -> Varchar,
        #[max_length = 128]
        from_nato_confidential -> Varchar,
        #[max_length = 128]
        from_nato_secret -> Varchar,
        #[max_length = 128]
        from_nato_top_secret -> Varchar,
        caveats -> Text,
        #[max_length = 32]
        version -> Varchar,
        authority_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        expires_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    data_objects (id) {
        id -> Uuid,
        creator_id -> Uuid,
        #[max_length = 512]
        title -> Varchar,
        description -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    metadata (id) {
        id -> Uuid,
        data_object_id -> Uuid,
        #[max_length = 256]
        domain -> Varchar,
        tags -> Array<Nullable<Text>>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    nations (id) {
        id -> Uuid,
        creator_id -> Uuid,
        #[max_length = 3]
        nation_code -> Varchar,
        #[max_length = 128]
        nation_name -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        hash -> Varchar,
        #[max_length = 128]
        email -> Varchar,
        #[max_length = 64]
        role -> Varchar,
        #[max_length = 256]
        name -> Varchar,
        #[max_length = 64]
        access_level -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        #[max_length = 256]
        access_key -> Varchar,
        approved_by_user_uid -> Nullable<Uuid>,
    }
}

diesel::table! {
    valid_roles (role) {
        #[max_length = 64]
        role -> Varchar,
    }
}

diesel::joinable!(authorities -> nations (nation_id));
diesel::joinable!(authorities -> users (creator_id));
diesel::joinable!(classification_schemas -> authorities (authority_id));
diesel::joinable!(classification_schemas -> users (creator_id));
diesel::joinable!(data_objects -> users (creator_id));
diesel::joinable!(metadata -> data_objects (data_object_id));
diesel::joinable!(nations -> users (creator_id));
diesel::joinable!(users -> valid_roles (role));

diesel::allow_tables_to_appear_in_same_query!(
    authorities,
    classification_schemas,
    data_objects,
    metadata,
    nations,
    users,
    valid_roles,
);
