use crate::{
    appstate::{AppState, IAppState},
    entities::friends::repository::IFriendRepository,
    helper::session::ISessionManager,
    persistence::connection_manager::IConnectionManager,
};
use core::time;
use std::sync::Arc;

use crate::helper::session::ISession;

pub fn initialize_session_cleanup_schedule<
    SM: ISessionManager<S, F>,
    S: ISession<F>,
    F: IFriendRepository,
    C: IConnectionManager,
>(
    app_state: Arc<AppState<SM, S, C, F>>,
) {
    let app_state = app_state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(time::Duration::from_secs(15));
        loop {
            interval.tick().await;
            app_state
                .current_user_connections
                .remove_expired_current_user_connections_sessions()
                .await
        }
    });
}
