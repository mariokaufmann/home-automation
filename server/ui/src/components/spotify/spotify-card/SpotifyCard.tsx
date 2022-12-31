import { Component } from "solid-js";
import { getPlaylists } from "../../../api/spotify";
import styles from "./SpotifyCard.module.css";

const SpotifyCard: Component = () => {
  return (
    <div class={styles.SpotifyCard}>
      <div class={styles.cardHeader}>
        {" "}
        <IconButton
          iconUrl={icon}
          backgroundColor={backgroundColor}
          iconColor={iconColor}
          onClick={onClick}
        ></IconButton>
      </div>
      <ul>
        {getPlaylists().map((playlist) => (
          <li>{playlist.name}</li>
        ))}
      </ul>
    </div>
  );
};

export default SpotifyCard;
