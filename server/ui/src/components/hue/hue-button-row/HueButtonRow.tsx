import { Component } from 'solid-js';
import styles from './HueButtonRow.module.css';
import IconButton from '../../icon-button/IconButton';
import offBulbIcon from 'bootstrap-icons/icons/lightbulb-off-fill.svg';
import bulbIcon from 'bootstrap-icons/icons/lightbulb-fill.svg';

const HueButtonRow: Component = () => {

  return (
    <div class={styles.HueButtonRow}>
      <IconButton iconUrl={offBulbIcon} iconColor="#EBE3E3" backgroundColor="#424242"></IconButton>
      <IconButton iconUrl={bulbIcon} iconColor="#EBE3E3" backgroundColor="#6F4B08"></IconButton>
      <IconButton iconUrl={bulbIcon} backgroundColor="#D29A2F"></IconButton>
      <IconButton iconUrl={bulbIcon} backgroundColor="#E9BF6F"></IconButton>
      <IconButton iconUrl={bulbIcon} backgroundColor="white"></IconButton>
    </div>
  );
};

export default HueButtonRow;