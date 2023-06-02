// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "video_status"))]
    pub struct VideoStatus;
}

diesel::table! {
    files (id) {
        id -> Uuid,
        file_name -> Varchar,
        file_extension -> Varchar,
        size -> Int8,
    }
}

diesel::table! {
    scheduled_archivals (id) {
        id -> Int4,
        url -> Varchar,
        scheduled_at -> Timestamp,
        not_before -> Timestamp,
    }
}

diesel::table! {
    tracked_collections (id) {
        id -> Int4,
        url -> Varchar,
        tracking_started_at -> Timestamp,
        last_checked -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::VideoStatus;

    videos (id) {
        id -> Int4,
        title -> Varchar,
        channel -> Varchar,
        views -> Int8,
        upload_date -> Timestamp,
        archived_date -> Timestamp,
        duration -> Int4,
        original_url -> Varchar,
        status -> VideoStatus,
        file_id -> Uuid,
        thumbnail_id -> Uuid,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    files,
    scheduled_archivals,
    tracked_collections,
    videos,
);
