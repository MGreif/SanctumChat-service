use std::time::Duration;

use axum::async_trait;
use tokio::sync::broadcast::Sender;

use crate::helper::session::ISessionManager;

#[derive(Debug, Clone)]
struct MockSessionManager {
    a: u8,
}

#[async_trait]
impl ISessionManager for MockSessionManager {
    fn get_token(&self) -> crate::helper::jwt::Token {
        return crate::helper::jwt::Token {
            exp: Duration::new(1000, 0),
            public_key: String::from("abc"),
            sub: String::from("Sub"),
        };
    }
    fn get_user(&self) -> crate::models::UserDTO {
        return crate::models::UserDTO {
            password: String::from("Pass"),
            username: String::from("User"),
            public_key: vec![{ '\x11' as u8 }],
        };
    }
    fn get_user_socket(
        &self,
    ) -> tokio::sync::broadcast::Sender<crate::handler::ws_handler::SocketMessage> {
        return Sender::new(1);
    }
    fn new(user: crate::models::UserDTO, token: crate::helper::jwt::Token) -> Self {
        return Self { a: '\x11' as u8 };
    }

    async fn notify_online(&self) {}
    async fn notify_offline(&self) {}
}

mod tests {
    use super::*;
    use crate::appstate;

    #[tokio::test]
    async fn test_something() {
        assert!("1" == "1");
    }
}
