import { Component } from 'solid-js';
import styles from './IconButton.module.css';

const IconButton: Component<{iconUrl: string, iconColor?: string, backgroundColor?: string}> = ({iconUrl, backgroundColor, iconColor}) => {
  const maskStyle = `mask-image: url(${iconUrl}); background-color: ${iconColor ?? 'black'}`;
  return (
    <div class={styles.IconButton}
         style={backgroundColor ? `background-color: ${backgroundColor}`: undefined}>
      <div class={styles.icon} style={maskStyle}>
      </div>
      {/*<img src={iconUrl} width="30" height="30"/>*/}
    </div>
  );
};

export default IconButton;