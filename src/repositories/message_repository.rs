use diesel::{r2d2::{PooledConnection, ConnectionManager}, PgConnection};
use diesel::prelude::*;
use diesel::query_dsl::*;
use crate::{models::Message, schema::messages::{sender, sent_at, all_columns, self, recipient}, helper::pagination::Pagination};


pub trait MessageRepositoryInterface {
    fn get_messages(&mut self, username: &String, origin: &String, pagination: Pagination) -> Result<Vec<Message>, String>;
    fn save_message(&mut self, message: &Message) -> Result<(), String>;
}

pub struct MessageRepository {
    pub pg_pool: PooledConnection<ConnectionManager<PgConnection>>
}

impl MessageRepositoryInterface for MessageRepository {
    fn get_messages(&mut self, username: &String, origin: &String, pagination: Pagination) -> Result<Vec<Message>, String> {
        let client_sent_or_received = sender.eq(username.clone()).or(recipient.eq(username.clone()));
        let recipient_sent_or_received = sender.eq(origin.clone()).or(recipient.eq(origin));

        let offset: i64 = (pagination.index * pagination.size).into();
        let limit: i64 = pagination.size.into();
        
        let sql_query = messages::table
            .select(all_columns)
            .order_by(sent_at.desc())
            .offset(offset);

        let db_messages = sql_query
            .limit(limit)
            .filter(client_sent_or_received)
            .filter(recipient_sent_or_received)
            .load(&mut self.pg_pool);

        let mut db_messages: Vec<Message> = match db_messages {
            Err(err) => return Err(format!("Could not get messages from db: {}", err)),
            Ok(res) => res 
        };

        db_messages.reverse();


        Ok(db_messages)
    }
    fn save_message(&mut self, message: &Message) -> Result<(), String> {
        let result = diesel::insert_into(messages::table).values(message).execute(&mut self.pg_pool);
        let result = match result {
            Err(err) => return Err(format!("Could not save message {:?}: {}", message, err)),
            Ok(res) => res
        };

        if result == 0 {
            return Err(String::from("Inserted 0 items, maybe mistake??"))
        };
        return Ok(())
    }
}