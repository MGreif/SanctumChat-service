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
        #[max_length = 1024]
        content -> Varchar,
        #[max_length = 1024]
        content_self_encrypted -> Varchar,
        #[max_length = 1024]
        content_signature -> Varchar,
        #[max_length = 1024]
        content_self_encrypted_signature -> Varchar,
    }
}

diesel::table! {
    users (username) {
        #[max_length = 30]
        username -> Varchar,
        #[max_length = 64]
        password -> Varchar,
        public_key -> Bytea,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    friend_requests,
    friends,
    messages,
    users,
);
