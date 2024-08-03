use crate::{
    entities::friends::repository::IFriendRepository,
    helper::session::{ISession, ISessionManager},
};

pub async fn check_active_sessions<
    SM: ISessionManager<S, F>,
    S: ISession<F>,
    F: IFriendRepository,
>(
    session_manager: SM,
) {
    let current_user_connections = session_manager.get_current_user_connections().lock().await;

    for (user, session_manager) in current_user_connections.iter() {
        
    }
}
