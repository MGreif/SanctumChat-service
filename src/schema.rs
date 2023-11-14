// @generated automatically by Diesel CLI.

diesel::table! {
    users (name) {
        #[max_length = 30]
        name -> Varchar,
        age -> Int4,
        #[max_length = 30]
        id -> Varchar,
        #[max_length = 64]
        password -> Varchar,
    }
}
