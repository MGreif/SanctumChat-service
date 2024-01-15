use crate::{repositories::message_repository::MessageRepositoryInterface, models::Message, helper::{errors::HTTPResponse, pagination::Pagination}};


pub struct MessageDomain<I: MessageRepositoryInterface> {
    message_repository: I
}

impl<I: MessageRepositoryInterface> MessageDomain<I> {
    pub fn new(message_repository: I) -> Self {
        return Self {
            message_repository
        }
    }

    pub fn get_messages(&mut self, username: &String, origin: &String, pagination: Pagination) -> Result<Vec<Message>, HTTPResponse<()>> {
        match self.message_repository.get_messages(username, origin, pagination) {
            Ok(res) => Ok(res),
            Err(err) => Err(HTTPResponse::new_internal_error(err))
        }
    }
}