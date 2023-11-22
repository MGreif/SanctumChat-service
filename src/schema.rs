// @generated automatically by Diesel CLI.

diesel::table! {
    friend_requests (id) {
        id -> Uuid,
        sender -> Uuid,
        recipient -> Uuid,
        accepted -> Nullable<Bool>,
    }
}

diesel::table! {
    friends (id) {
        id -> Uuid,
        user_a -> Uuid,
        user_b -> Uuid,
    }
}

diesel::table! {
    messages (id) {
        id -> Uuid,
        sender -> Uuid,
        recipient -> Uuid,
        sent_at -> Timestamp,
        #[max_length = 500]
        content -> Varchar,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 30]
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
