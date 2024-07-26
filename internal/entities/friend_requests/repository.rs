use crate::{
    entities::friends::controller::FriendRequestGETResponseDTO,
    models::FriendRequest,
    schema::{
        friend_requests::{self},
        users::dsl::*,
    },
};
use diesel::prelude::*;
use diesel::query_dsl::*;
use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    sql_types::{Bool, Text, Uuid},
    PgConnection,
};

pub trait FriendRequestRepositoryInterface {
    fn get_friend_requests_sent_to_user(
        &mut self,
        usern: &String,
    ) -> Result<Vec<FriendRequestGETResponseDTO>, String>;
    fn check_if_friend_request_is_present(
        &mut self,
        sender: &String,
        recipient: &String,
        accepted: Option<bool>,
    ) -> Result<bool, String>;
    fn save_friend_request(&mut self, friend_request: FriendRequest) -> Result<(), String>;
    fn update_friend_request_accepted(
        &mut self,
        friend_request_id: &uuid::Uuid,
        recipient: &String,
        accepted: bool,
    ) -> Result<(), String>;
}

pub struct FriendRequestRepository {
    pub pg_pool: PooledConnection<ConnectionManager<PgConnection>>,
}

impl FriendRequestRepositoryInterface for FriendRequestRepository {
    fn get_friend_requests_sent_to_user(
        &mut self,
        usern: &String,
    ) -> Result<Vec<FriendRequestGETResponseDTO>, String> {
        let query = diesel::sql_query("SELECT r.id as id, u.username as sender_id, r.recipient as recipient, r.accepted as accepted FROM friend_requests as r INNER JOIN users as u ON u.username = r.sender WHERE r.recipient = $1 AND r.accepted IS NULL")
            .bind::<diesel::sql_types::Text, _>(usern);
        let result = query.load(&mut self.pg_pool);
        match result {
            Ok(res) => Ok(res),
            Err(err) => Err(format!("Could not get friend_requests: {}", err)),
        }
    }

    fn check_if_friend_request_is_present(
        &mut self,
        sender: &String,
        recipient: &String,
        accepted: Option<bool>,
    ) -> Result<bool, String> {
        let mut addon = "AND accepted IS NULL";
        if let Some(accepted) = accepted {
            if !accepted {
                addon = "AND accepted IS FALSE"
            } else {
                addon = "AND accepted IS TRUE"
            }
        }

        let count = diesel::sql_query("SELECT COUNT(*) FROM friend_requests WHERE (sender = $1 AND recipient = $2) OR (sender = $2 AND recipient = $1) ".to_owned() + addon)
            .bind::<diesel::sql_types::Text, _>(sender)
            .bind::<Text, _>(recipient)
            .load::<crate::helper::sql::Count>(&mut self.pg_pool);

        let mut count = match count {
            Ok(c) => c,
            Err(err) => return Err(format!("Could not get friend requests: {}", err)),
        };

        let count = match count.pop() {
            Some(i) => i.count,
            None => return Err(format!("Could not get present friend-requests count")),
        };

        return Ok(count > 0);
    }

    fn save_friend_request(&mut self, friend_request: FriendRequest) -> Result<(), String> {
        let inserted_rows = match diesel::insert_into(friend_requests::table)
            .values(&vec![friend_request])
            .execute(&mut self.pg_pool)
        {
            Ok(t) => t,
            Err(err) => return Err(format!("Could not insert friend request: {:?}", err)),
        };

        if inserted_rows == 0 {
            return Err(String::from("0 rows inserted ... maybe a problem?"));
        }

        return Ok(());
    }

    fn update_friend_request_accepted(
        &mut self,
        friend_request_id: &uuid::Uuid,
        recipient: &String,
        accepted: bool,
    ) -> Result<(), String> {
        let mut query = diesel::sql_query("UPDATE friend_requests SET ").into_boxed();

        query = query.sql("accepted = $1 ").bind::<Bool, _>(accepted);

        let query = query
            .sql("WHERE id = $2 AND recipient = $3")
            .bind::<diesel::sql_types::Uuid, _>(friend_request_id)
            .bind::<diesel::sql_types::Text, _>(recipient);
        let patched = query.execute(&mut self.pg_pool);
        let patched = match patched {
            Err(err) => return Err(format!("Could not patch friend request: {}", err)),
            Ok(res) => res,
        };

        if patched == 0 {
            return Err(String::from("0 rows patched ... maybe a problem?"));
        }

        return Ok(());
    }
}
