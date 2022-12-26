import { createSignal } from "solid-js";
import { get, put } from "./api";
import { PhilipsHueGroupDto } from "../types/PhilipsHueGroupDto";
import { PhilipsHuePresetDto } from "../types/PhilipsHuePresetDto";
import { PhilipsHueConfigureGroupDto } from "../types/PhilipsHueConfigureGroupDto";

const [getGroups, setGroups] = createSignal<PhilipsHueGroupDto[]>([]);
const [getPresets, setPresets] = createSignal<PhilipsHuePresetDto[]>([]);

// fetch data
get<PhilipsHueGroupDto[]>("philipshue/groups").then((groups) =>
  setGroups(groups)
);
get<PhilipsHuePresetDto[]>("philipshue/presets").then((presets) =>
  setPresets(presets)
);

export async function configureGroupedLight(group: string, preset: string) {
  let payload: PhilipsHueConfigureGroupDto = {
    group_id: group,
    preset_id: preset,
  };
  await put("philipshue/groups", payload);
}

export { getGroups, getPresets };
