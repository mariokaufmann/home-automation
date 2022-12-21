import { Component } from 'solid-js';
import HueButtonRow from '../hue-button-row/HueButtonRow';
import styles from './HueCard.module.css';

const HueCard: Component = () => {

  return (
    <div class={styles.HueCard}>
      <h2>Living Room</h2>
      <HueButtonRow></HueButtonRow>
      <h2>Kitchen</h2>
      <HueButtonRow></HueButtonRow>
      <h2>TV</h2>
      <HueButtonRow></HueButtonRow>
    </div>
  );
};

export default HueCard;