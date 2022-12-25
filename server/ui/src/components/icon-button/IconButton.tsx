import { Component } from "solid-js";
import styles from "./IconButton.module.css";

const IconButton: Component<{
  iconUrl: string;
  iconColor?: string;
  backgroundColor?: string;
  onClick: () => void;
}> = ({ iconUrl, backgroundColor, iconColor, onClick }) => {
  const maskStyle = `mask-image: url(${iconUrl}); background-color: ${
    iconColor ?? "black"
  }`;
  return (
    <button
      class={styles.IconButton}
      style={
        backgroundColor ? `background-color: ${backgroundColor}` : undefined
      }
      onClick={onClick}
    >
      <div class={styles.icon} style={maskStyle}></div>
    </button>
  );
};

export default IconButton;
