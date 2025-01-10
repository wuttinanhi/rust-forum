// @generated automatically by Diesel CLI.

diesel::table! {
    comments (id) {
        id -> Int4,
        content -> Text,
        post_id -> Int4,
        user_id -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    posts (id) {
        id -> Int4,
        title -> Varchar,
        body -> Text,
        published -> Bool,
        user_id -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        user_profile_picture_url -> Nullable<Varchar>,
        #[max_length = 255]
        role -> Varchar,
    }
}

diesel::joinable!(comments -> posts (post_id));
diesel::joinable!(comments -> users (user_id));
diesel::joinable!(posts -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    comments,
    posts,
    users,
);
