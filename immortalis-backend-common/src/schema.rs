// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "video_status"))]
    pub struct VideoStatus;
}

diesel::table! {
    downloads (id) {
        id -> Int4,
        video_id -> Int4,
        title -> Varchar,
        value -> Varchar,
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
        thumbnail_address -> Varchar,
        original_url -> Varchar,
        status -> VideoStatus,
    }
}

diesel::joinable!(downloads -> videos (video_id));

diesel::allow_tables_to_appear_in_same_query!(
    downloads,
    scheduled_archivals,
    videos,
);
