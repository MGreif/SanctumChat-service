use diesel::prelude::*;

#[derive(Debug, serde::Deserialize, serde::Serialize, diesel::Queryable, diesel::Selectable, diesel::Insertable)]
#[diesel(table_name = crate::schema::root)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RootDTO {
    pub name: String
}