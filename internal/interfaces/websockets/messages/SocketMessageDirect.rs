use crate::appstate::{AppState, IAppState};
use crate::entities::friends::repository::IFriendRepository;
use crate::helper::session::ISessionManager;
use crate::interfaces::websockets::socket_messages::{
    Receivable, SocketMessage, SocketMessageError,
};
use crate::persistence::connection_manager::IConnectionManager;
use crate::{
    entities::{
        friends::{repository::FriendRepository, service::FriendDomain},
        messages::{messages::MessageDomain, repository::MessageRepository},
    },
    helper::{jwt::Token, session::ISession},
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct SocketMessageDirect {
    pub recipient: Option<String>,
    pub sender: Option<String>,
    pub message: String,
    pub message_signature: String,
    pub message_self_encrypted: String,
    pub message_self_encrypted_signature: String,
    pub id: Option<Uuid>,
    pub TYPE: Option<String>,
}

impl SocketMessageDirect {
    pub fn new(
        sender: Option<String>,
        recipient: Option<String>,
        message: String,
        message_signature: String,
        message_self_encrypted: String,
        message_self_encrypted_signature: String,
    ) -> SocketMessageDirect {
        SocketMessageDirect {
            message,
            message_signature,
            message_self_encrypted,
            message_self_encrypted_signature,
            id: Some(Uuid::new_v4()),
            recipient,
            sender,
            TYPE: Some(String::from("SOCKET_MESSAGE_DIRECT")),
        }
    }
}

impl<SM: ISessionManager<S, F>, S: ISession<F>, F: IFriendRepository, C: IConnectionManager>
    Receivable<SM, S, F, C> for SocketMessageDirect
{
    async fn handle_receive(
        &self,
        app_state: Arc<AppState<SM, S, C, F>>,
        token: Token,
    ) -> Result<(), SocketMessageError> {
        let message_repo = MessageRepository {
            pg_pool: app_state.get_db_pool(),
        };

        let friend_repo = FriendRepository {
            pg_pool: C::new(app_state.get_config().env),
        };

        let friend_domain = FriendDomain::new(friend_repo);
        let mut message_domain = MessageDomain::new(message_repo);
        let recipient = match &self.recipient {
            None => {
                return Err(SocketMessageError::new(String::from(
                    "No recipient specified",
                )))
            }
            Some(r) => r,
        };

        let has_friend = match friend_domain.check_if_user_has_friend(&token.sub, recipient) {
            Ok(res) => res,
            Err(_) => {
                return Err(SocketMessageError::new(String::from(
                    "Uuups, something went wrong..",
                )))
            }
        };

        if has_friend == false {
            return Err(SocketMessageError::new(format!(
                "You are not befriended with {}",
                recipient
            )));
        }

        // Get fresh connection to get latest state
        let client_session = app_state.get_session_manager().get_current_user_connections().lock().await.get(&token.sub).expect("Error getting client session. This should not appear because a session in create on login/token validations").lock().await.clone();

        let direct_message = SocketMessageDirect::new(
            Some(token.sub),
            self.recipient.clone(),
            self.message.clone(),
            self.message_signature.clone(),
            self.message_self_encrypted.clone(),
            self.message_self_encrypted_signature.clone(),
        );

        let message = message_domain.direct_message_to_message_entity(&direct_message);
        let message = match message {
            Ok(m) => m,
            Err(err) => {
                return Err(SocketMessageError::new(err))
            }
        };

        match message_domain.save_message(&message) {
            Err(err) => {
                tracing::error!("{}", &err);
                return Err(SocketMessageError::new(String::from("An error ocurred while saving the message ...")))
            },
            Ok(_) => {}
        };

        client_session
            .send_direct_message(SocketMessage::SocketMessageDirect(direct_message.clone()))
            .await;

        let current_user_connections = app_state
            .current_user_connections
            .get_current_user_connections()
            .lock()
            .await
            .clone();
        let recipient_session_manager = current_user_connections.get(recipient).clone();
        match recipient_session_manager {
            None => {}
            Some(sm) => {
                sm.lock()
                    .await
                    .send_direct_message(SocketMessage::SocketMessageDirect(direct_message.clone()))
                    .await;
            }
        }
        return Ok(());
    }
}
