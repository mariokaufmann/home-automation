import { createSignal } from "solid-js";
import { SpotifyPlaylistDto } from "../types/SpotifyPlaylistDto";
import { get } from "./api";

const [getPlaylists, setPlaylists] = createSignal<SpotifyPlaylistDto[]>([]);

get<SpotifyPlaylistDto[]>("spotify/playlists").then((playlists) =>
  setPlaylists(playlists)
);

export { getPlaylists };
