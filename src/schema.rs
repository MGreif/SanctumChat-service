// @generated automatically by Diesel CLI.

diesel::table! {
    messages (id) {
        id -> Uuid,
        sender -> Uuid,
        recipient -> Uuid,
        sent_at -> Date,
    }
}

diesel::table! {
    users (id) {
        #[max_length = 30]
        name -> Varchar,
        age -> Int4,
        id -> Uuid,
        #[max_length = 64]
        password -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    messages,
    users,
);
