use crate::automodule::spotify::routes::auth::{check_auth, CheckAuthResult, SpotifyAuthState};
use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use axum::Json;

use home_automation_common::config::ConfigurationManager;
use hyper::StatusCode;
use std::sync::Arc;
use tokio::sync::Mutex;
use ts_rs::TS;

use super::api::{GetPlaylistDto, SpotifyApiClient};
use super::config::SpotifyAutomationModuleCachedConfiguration;

pub mod auth;

pub struct SpotifyState {
    api_client: SpotifyApiClient,
    auth_state: SpotifyAuthState,
}

impl SpotifyState {
    pub fn new(
        cached_configuration_manager: ConfigurationManager<
            SpotifyAutomationModuleCachedConfiguration,
        >,
    ) -> Self {
        SpotifyState {
            api_client: SpotifyApiClient::new(),
            auth_state: SpotifyAuthState::new(cached_configuration_manager),
        }
    }
}

#[derive(Serialize, TS)]
pub struct SpotifyPlaylistDto {
    name: String,
}

pub async fn get_playlists(
    State(state): State<Arc<Mutex<SpotifyState>>>,
) -> Result<impl IntoResponse, StatusCode> {
    Ok(Json(vec![
        SpotifyPlaylistDto {
            name: "Playlist 1".to_owned(),
        },
        SpotifyPlaylistDto {
            name: "Playlist 2".to_owned(),
        },
    ]))
    /*
    let mut locked_state = state.lock().await;
    let check_auth_result = check_auth(&mut locked_state.auth_state).await;
    match check_auth_result {
        CheckAuthResult::AuthValid { access_token } => {
            let playlists = locked_state
                .api_client
                .get_playlists(access_token.secret())
                .await;
            // TODO is there a better pattern, can we use ? here?
            match playlists {
                Ok(playlists) => {
                    let result: Vec<SpotifyPlaylistDto> = playlists
                        .into_iter()
                        .map(|playlist| SpotifyPlaylistDto {
                            name: playlist.name,
                        })
                        .collect();
                    Ok(Json(result).into_response())
                }
                Err(err) => {
                    error!("Could not load Spotify playlists: {}.", err);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        CheckAuthResult::NoAuth { login_url } => {
            let response = (StatusCode::OK, [("X-AUTOMATION-LOGIN", login_url)]);
            Ok(response.into_response())
        }
    }*/
}

#[cfg(test)]
mod test {

    use home_automation_common::types::export_type;

    use super::SpotifyPlaylistDto;

    #[test]
    fn export_types() {
        export_type::<SpotifyPlaylistDto>();
    }
}
