// https://de.wikipedia.org/wiki/Mired
export function mirekToKelvin(mirek: number): number {
  return 1_000_000 / mirek;
}

const clamp = (num: number, min: number, max: number): number =>
  Math.min(Math.max(num, min), max);

// https://tannerhelland.com/2012/09/18/convert-temperature-rgb-algorithm-code.html
export function kelvinToRgb(kelvin: number): [number, number, number] {
  let temperature = kelvin / 100;

  let red: number;
  if (temperature <= 66) {
    red = 255;
  } else {
    red = temperature - 60;
    red = 329.698727446 * Math.pow(red, -0.1332047592);
    red = clamp(red, 0, 255);
  }

  let green;
  if (temperature <= 66) {
    green = temperature;
    green = 99.4708025861 * Math.log(green) - 161.1195681661;
  } else {
    green = temperature - 60;
    green = 288.1221695283 * Math.pow(green, -0.0755148492);
  }
  green = clamp(green, 0, 255);

  let blue;
  if (temperature >= 66) {
    blue = 255;
  } else if (temperature <= 19) {
    blue = 0;
  } else {
    blue = temperature - 10;
    blue = 138.5177312231 * Math.log(blue) - 305.0447927307;
    blue = clamp(blue, 0, 255);
  }

  return [red, green, blue];
}
