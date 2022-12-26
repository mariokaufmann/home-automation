import { Component } from "solid-js";
import styles from "./HueButtonRow.module.css";
import { getPresets } from "../../../api/philipshue";
import HueButton from "../hue-button/HueButton";

const HueButtonRow: Component<{
  presetSelected: (presetId: string) => void;
}> = (props) => {
  return (
    <div class={styles.HueButtonRow}>
      {getPresets().map((preset) => (
        <HueButton
          preset={preset}
          onClick={() => props.presetSelected(preset.id)}
        ></HueButton>
      ))}
    </div>
  );
};

export default HueButtonRow;
