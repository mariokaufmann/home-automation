import { Component } from "solid-js";
import HueButtonRow from "../hue-button-row/HueButtonRow";
import styles from "./HueCard.module.css";
import { configureGroupedLight, getGroups } from "../../../api/philipshue";

const HueCard: Component = () => {
  return (
    <div class={styles.HueCard}>
      {getGroups().map((group) => (
        <>
          <h2>{group.name}</h2>
          <HueButtonRow
            presetSelected={(presetId) => configureGroupedLight(group.id, presetId)}
          ></HueButtonRow>
        </>
      ))}
    </div>
  );
};

export default HueCard;
