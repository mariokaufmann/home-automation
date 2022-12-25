import { Component } from "solid-js";
import { PhilipsHuePresetDto } from "../../../types/PhilipsHuePresetDto";
import { kelvinToRgb, mirekToKelvin } from "../color";
import IconButton from "../../icon-button/IconButton";
import bulbIcon from "bootstrap-icons/icons/lightbulb-fill.svg";
import offBulbIcon from "bootstrap-icons/icons/lightbulb-off-fill.svg";

const HueButton: Component<{
  preset: PhilipsHuePresetDto;
  onClick: () => void;
}> = ({ preset, onClick }) => {
  const rgb = kelvinToRgb(mirekToKelvin(preset.color_temperature));
  const backgroundColor = preset.on
    ? `rgb(${rgb[0]},${rgb[1]},${rgb[2]}`
    : "black";
  const iconColor = preset.on ? "black" : "white";
  const icon = preset.on ? bulbIcon : offBulbIcon;
  return (
    <IconButton
      iconUrl={icon}
      backgroundColor={backgroundColor}
      iconColor={iconColor}
      onClick={onClick}
    ></IconButton>
  );
};

export default HueButton;
