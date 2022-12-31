use crate::automodule::spotify::api::requester::ApiRequester;

mod requester;

#[derive(Deserialize)]
pub struct PaginatedResultsDto<T> {
    items: Vec<T>,
}

#[derive(Deserialize)]
pub struct GetPlaylistDto {
    pub name: String,
}

pub struct SpotifyApiClient {
    requester: ApiRequester,
}

impl SpotifyApiClient {
    pub fn new() -> Self {
        SpotifyApiClient {
            requester: ApiRequester::new(),
        }
    }

    pub async fn get_playlists(&self, access_token: &str) -> anyhow::Result<Vec<GetPlaylistDto>> {
        let response = self
            .requester
            .get::<PaginatedResultsDto<GetPlaylistDto>>("/me/playlists", access_token)
            .await?;
        Ok(response.items)
    }
}
