// @generated automatically by Diesel CLI.

diesel::table! {
    friend_requests (id) {
        id -> Uuid,
        #[max_length = 30]
        sender -> Varchar,
        #[max_length = 30]
        recipient -> Varchar,
        accepted -> Nullable<Bool>,
    }
}

diesel::table! {
    friends (id) {
        id -> Uuid,
        #[max_length = 30]
        user_id -> Varchar,
        #[max_length = 30]
        befriended_user_id -> Varchar,
    }
}

diesel::table! {
    messages (id) {
        id -> Uuid,
        #[max_length = 30]
        sender -> Varchar,
        #[max_length = 30]
        recipient -> Varchar,
        sent_at -> Timestamp,
        #[max_length = 500]
        content -> Varchar,
    }
}

diesel::table! {
    users (username) {
        #[max_length = 30]
        username -> Varchar,
        #[max_length = 50]
        name -> Varchar,
        age -> Int4,
        #[max_length = 64]
        password -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    friend_requests,
    friends,
    messages,
    users,
);
