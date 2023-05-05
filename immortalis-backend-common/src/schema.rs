// @generated automatically by Diesel CLI.

diesel::table! {
    downloads (id) {
        id -> Int4,
        video_id -> Int4,
        title -> Varchar,
        value -> Varchar,
    }
}

diesel::table! {
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
    }
}

diesel::joinable!(downloads -> videos (video_id));

diesel::allow_tables_to_appear_in_same_query!(
    downloads,
    videos,
);
