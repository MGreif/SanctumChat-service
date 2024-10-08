use crate::appstate::{AppState, IAppState};
use crate::entities::friends::repository::IFriendRepository;
use crate::helper::errors::HTTPResponse;
use crate::helper::session::{ISession, ISessionManager};
use crate::persistence::connection_manager::IConnectionManager;
use crate::validation::string_validate::DEFAULT_INPUT_FIELD_STRING_VALIDATOR;
use crate::{
    helper::{
        jwt::{hash_string, Token},
        keys::{generate_rsa_key_pair, validate_public_key},
    },
    models::UserDTO,
};
use axum::{
    extract::{Json, State},
    http::{
        header::{self, SET_COOKIE},
        HeaderMap, StatusCode,
    },
    response::IntoResponse,
    Extension,
};
use base64;
use base64::Engine;
use std::sync::Arc;

use super::repository::UserRepository;
use super::users::UserDomain;

#[derive(serde::Deserialize)]
pub struct UserCreateDTO {
    pub username: String,
    password: String,
    pub public_key: String,
    pub generate_key: bool,
}

#[derive(serde::Deserialize)]
pub struct GetUserQueryDTO {
    pub name: Option<String>,
}

pub async fn create_user<
    'a,
    SM: ISessionManager<S, F>,
    S: ISession<F>,
    F: IFriendRepository,
    C: IConnectionManager,
>(
    State(state): State<Arc<AppState<SM, S, C, F>>>,
    Json(body): Json<UserCreateDTO>,
) -> impl IntoResponse {
    let user_repository = UserRepository {
        pg_pool: state.get_db_pool(),
    };
    let mut user_domain = UserDomain::new(user_repository);

    let headers = HeaderMap::new();

    match DEFAULT_INPUT_FIELD_STRING_VALIDATOR.validate(&body.username) {
        Err(err) => {
            return (
                headers,
                HTTPResponse::<Vec<u8>> {
                    message: Some(format!("Username validation failed: {}", err)),
                    data: None,
                    status: StatusCode::BAD_REQUEST,
                },
            )
        }
        Ok(_) => {}
    }

    match DEFAULT_INPUT_FIELD_STRING_VALIDATOR.validate(&body.password) {
        Err(err) => {
            return (
                headers,
                HTTPResponse {
                    message: Some(format!("Password validation failed: {}", err)),
                    data: None,
                    status: StatusCode::BAD_REQUEST,
                },
            )
        }
        Ok(_) => {}
    }

    let mut private_key: Option<Vec<u8>> = None;
    let mut pub_key: Vec<u8> = body.public_key.as_bytes().to_vec().clone();

    let pub_key_decoded = base64::engine::general_purpose::STANDARD.decode(body.public_key);
    let pub_key_decoded = match pub_key_decoded {
        Ok(r) => r,
        Err(err) => {
            return (
                headers,
                HTTPResponse {
                    message: Some(format!("Public key encoding failed: {}", err)),
                    data: None,
                    status: StatusCode::BAD_REQUEST,
                },
            )
        }
    };

    /*     #let pub_key_decoded = match from_utf8(&pub_key_decoded) {
        Ok(r) => r,
        Err(err) => return (headers, HTTPResponse {
            message: Some(format!("Public key encoding failed: {}", err)),
            data: None,
            status: StatusCode::BAD_REQUEST
        })
    }; */

    if body.generate_key == false {
        match validate_public_key(&pub_key_decoded) {
            Err(_) => return (
                headers,
                HTTPResponse {
                    data: None,
                    message: Some(String::from(
                        "Could not validate public key. Ensure that its using .PEM PKCS#8 format",
                    )),
                    status: StatusCode::BAD_REQUEST,
                },
            ),
            Ok(_) => {}
        }
    }

    if body.generate_key == true {
        let (rsa_private_key, rsa_public_key) = match generate_rsa_key_pair() {
            Ok(res) => res,
            Err(err) => {
                return (
                    headers,
                    HTTPResponse {
                        data: None,
                        message: Some(err),
                        status: StatusCode::INTERNAL_SERVER_ERROR,
                    },
                )
            }
        };

        private_key = Some(rsa_private_key);

        let output = base64::engine::general_purpose::STANDARD.encode(rsa_public_key.as_slice());
        pub_key = output.as_bytes().to_vec();
    };

    let new_user = UserDTO {
        username: body.username,
        password: body.password,
        public_key: pub_key,
    };

    let result = user_domain.create_user(&new_user, &state.get_config().env.HASHING_KEY.as_bytes());

    match result {
        Ok(_) => (
            headers,
            HTTPResponse {
                message: Some(format!("User created successfully")),
                data: private_key,
                status: StatusCode::CREATED,
            },
        ),
        Err(err) => (headers, err),
    }
}

#[derive(serde::Deserialize)]
pub struct LoginDTO {
    pub username: String,
    pub password: String,
}

pub async fn logout<
    SM: ISessionManager<S, F>,
    S: ISession<F>,
    F: IFriendRepository,
    C: IConnectionManager,
>(
    State(state): State<Arc<AppState<SM, S, C, F>>>,
    token: Extension<Token>,
) -> impl IntoResponse {
    let session = match state
        .get_session_manager()
        .remove_from_current_user_connections(&token.sub)
        .await
    {
        Ok(sm) => sm,
        Err(err) => {
            return HTTPResponse::<()> {
                message: Some(err),
                data: None,
                status: StatusCode::INTERNAL_SERVER_ERROR,
            }
        }
    };

    session
        .lock()
        .await
        .notify_offline(state.get_session_manager())
        .await;

    tracing::debug!(target: "application", "[logout] user {} manually logged out", token.sub);
    HTTPResponse::<()> {
        message: Some(String::from("Successfully logged out")),
        data: None,
        status: StatusCode::OK,
    }
}

pub async fn login<
    SM: ISessionManager<S, F>,
    S: ISession<F>,
    F: IFriendRepository,
    C: IConnectionManager,
>(
    State(state): State<Arc<AppState<SM, S, C, F>>>,
    Json(body): Json<LoginDTO>,
) -> impl IntoResponse {
    let LoginDTO {
        password: pw,
        username: username_id,
    } = body;
    let user_repository = UserRepository {
        pg_pool: state.get_db_pool(),
    };
    let mut user_domain = UserDomain::new(user_repository);

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());

    match DEFAULT_INPUT_FIELD_STRING_VALIDATOR.validate(&username_id) {
        Err(err) => {
            return (
                headers,
                HTTPResponse::<()> {
                    data: None,
                    message: Some(String::from(format!("Username validation failed: {}", err))),
                    status: StatusCode::UNAUTHORIZED,
                }
                .into_response(),
            )
        }
        Ok(_) => {}
    }

    match DEFAULT_INPUT_FIELD_STRING_VALIDATOR.validate(&pw) {
        Err(err) => {
            return (
                headers,
                HTTPResponse::<()> {
                    data: None,
                    message: Some(String::from(format!("Password validation failed: {}", err))),
                    status: StatusCode::UNAUTHORIZED,
                }
                .into_response(),
            )
        }
        Ok(_) => {}
    }

    let pw = hash_string(&pw, state.get_config().env.HASHING_KEY.as_bytes());
    let token = user_domain.login_user_and_prepare_token(
        &username_id,
        &pw,
        state.get_config().env.HASHING_KEY.as_bytes(),
    );

    let (user, token, session_token) = match token {
        Ok(result) => result,
        Err(err) => return (headers, err.into_response()),
    };

    let session = S::new(user.clone(), token);
    session.notify_online(state.get_session_manager()).await;
    state
        .get_session_manager()
        .insert_into_current_user_connections(session)
        .await;

    headers.insert(
        SET_COOKIE,
        format!(
            "session={}; Max-Age=2592000; Path=/; SameSite=None",
            session_token
        )
        .parse()
        .unwrap(),
    );

    (
        headers,
        HTTPResponse::<String> {
            data: Some(session_token),
            message: Some(String::from("Login successful")),
            status: StatusCode::OK,
        }
        .into_response(),
    )
}

pub async fn token<
    'a,
    SM: ISessionManager<S, F>,
    F: IFriendRepository,
    S: ISession<F>,
    T: IAppState<F, SM, S>,
>(
    State(app_state): State<Arc<T>>,
    Extension(token): Extension<Token>,
) -> impl IntoResponse {
    let pool = app_state.get_db_pool();
    let repository = UserRepository { pg_pool: pool };
    let mut domain = UserDomain::new(repository);

    let result = domain.renew_token(
        &token.sub,
        app_state.get_config().env.HASHING_KEY.as_bytes(),
    );

    let (user, token, token_str) = match result {
        Ok(result) => result,
        Err(err) => {
            return HTTPResponse::<()> {
                status: StatusCode::BAD_REQUEST,
                data: None,
                message: Some(err),
            }
            .into_response()
        }
    };

    let user_id = &token.sub;

    let available_session = app_state.get_session_manager().get_current_user_connections().lock().await;
    let session = available_session.get(user_id);

    if let Some(_) = session {
        return     HTTPResponse::<String> {
            data: Some(token_str),
            message: None,
            status: StatusCode::OK,
        }
        .into_response()
    }

    drop(available_session);

    let session = S::new(user.clone(), token);
    session.notify_online(app_state.get_session_manager()).await;
    app_state
        .get_session_manager()
        .insert_into_current_user_connections(session)
        .await;

    HTTPResponse::<String> {
        data: Some(token_str),
        message: None,
        status: StatusCode::OK,
    }
    .into_response()
}
