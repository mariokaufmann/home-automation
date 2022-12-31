use crate::automodule::spotify::config::SpotifyAutomationModuleCachedConfiguration;
use crate::automodule::spotify::routes::auth::CheckAuthResult::{AuthValid, NoAuth};
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect};
use home_automation_common::config::ConfigurationManager;
use hyper::StatusCode;
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AccessToken, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, RefreshToken, Scope, TokenResponse, TokenUrl,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::SpotifyState;

pub struct SpotifyAuthState {
    access_token: Option<AccessToken>,
    cached_configuration_manager: ConfigurationManager<SpotifyAutomationModuleCachedConfiguration>,
    client: BasicClient,
    current_auth_request: Option<CurrentAuth>,
}

pub struct CurrentAuth {
    csrf_token: CsrfToken,
    pkce_verifier: PkceCodeVerifier,
}

impl SpotifyAuthState {
    pub fn new(
        cached_configuration_manager: ConfigurationManager<
            SpotifyAutomationModuleCachedConfiguration,
        >,
    ) -> Self {
        // TODO unwrap
        let client = BasicClient::new(
            ClientId::new("16bce770a4de4bbea26c316c9b269f49".to_string()),
            Some(ClientSecret::new(
                "066bff6b94ed42b79f81d5139518e6c0".to_string(),
            )),
            AuthUrl::new("https://accounts.spotify.com/authorize".to_string()).unwrap(),
            Some(TokenUrl::new("https://accounts.spotify.com/api/token".to_string()).unwrap()),
        )
        .set_redirect_uri(
            RedirectUrl::new("http://localhost:3000/api/spotify/auth/callback".to_string())
                .unwrap(),
        );

        SpotifyAuthState {
            client,
            access_token: None,
            current_auth_request: None,
            cached_configuration_manager,
        }
    }
}

pub enum CheckAuthResult {
    AuthValid { access_token: AccessToken },
    NoAuth { login_url: String },
}

// checks whether we are currently logged into spotify and returns the redirect url if not
pub async fn check_auth(state: &mut SpotifyAuthState) -> CheckAuthResult {
    let cached_configuration = state.cached_configuration_manager.get_configuration();

    if let Some(access_token) = &state.access_token {
        debug!("Spotify access token present.");
        AuthValid {
            access_token: access_token.clone(),
        }
    } else if let Some(refresh_token) = &cached_configuration.refresh_token {
        debug!("Refreshing Spotify access token with refresh token.");
        let refresh_token = RefreshToken::new(refresh_token.clone());
        match state
            .client
            .exchange_refresh_token(&refresh_token)
            .request_async(async_http_client)
            .await
        {
            Ok(token_result) => {
                state.access_token = Some(token_result.access_token().clone());
                info!("access token: {}", token_result.access_token().secret());
                AuthValid {
                    access_token: token_result.access_token().clone(),
                }
            }
            Err(err) => {
                error!("Could not refresh Spotify token: {}.", err);
                let redirect_url = get_auth_url_for_new_flow(state);
                NoAuth {
                    login_url: redirect_url,
                }
            }
        }
    } else {
        let redirect_url = get_auth_url_for_new_flow(state);
        NoAuth {
            login_url: redirect_url,
        }
    }
}

fn get_auth_url_for_new_flow(state: &mut SpotifyAuthState) -> String {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token) = state
        .client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("playlist-read-private".to_string()))
        .add_scope(Scope::new("user-read-playback-state".to_string()))
        .add_scope(Scope::new("user-modify-playback-state".to_string()))
        .add_scope(Scope::new("user-read-currently-playing".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    state.current_auth_request = Some(CurrentAuth {
        pkce_verifier,
        csrf_token,
    });

    auth_url.to_string()
}

#[derive(Deserialize)]
pub struct AuthCallbackQuery {
    code: String,
    state: String,
}

pub async fn callback_auth(
    query: Query<AuthCallbackQuery>,
    State(state): State<Arc<Mutex<SpotifyState>>>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut locked_state = state.lock().await;
    let auth_state = &mut locked_state.auth_state;

    // TODO unwrap
    let current_auth = auth_state.current_auth_request.take().unwrap();
    if !current_auth.csrf_token.secret().eq(&query.state) {
        warn!("State parameter does not match.");
        return Err(StatusCode::BAD_REQUEST);
    }

    let token_result = auth_state
        .client
        .exchange_code(AuthorizationCode::new(query.code.clone()))
        .set_pkce_verifier(current_auth.pkce_verifier)
        .request_async(async_http_client)
        .await
        .unwrap();

    auth_state.access_token = Some(token_result.access_token().clone());
    info!("access token: {}", token_result.access_token().secret());

    if let Some(refresh_token) = token_result.refresh_token() {
        let config = auth_state
            .cached_configuration_manager
            .get_configuration_mut();
        config.refresh_token = Some(refresh_token.secret().clone());
        auth_state
            .cached_configuration_manager
            .persist_configuration()
            .unwrap();
    } else {
        warn!("No refresh token was returned.");
    }

    Ok(Redirect::to("http://localhost:3000"))
}
