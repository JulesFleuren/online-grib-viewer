import {
  find_min_max_value,
  find_min_max_magnitude,
} from "../pkg/online_grib_viewer.js";
type GribKey = import("./gribKey.js").GribKey;

interface HeatMapOverlaySettings {
  colorMin: Number | null;
  colorMax: Number | null;
  removeOutOfBounds: boolean;
  pixelsPerPoint: number;
}

function createDefaultHeatMapOverlaySettings(
  gribBytes: Uint8Array,
  key: GribKey,
): HeatMapOverlaySettings {
  if (key.isVectorField) {
    const res = find_min_max_magnitude(
      gribBytes,
      key.firstComponent,
      key.secondComponent!,
    );
    const [minMagnitude, maxMagnitude] = [res.min, res.max];
    return {
      colorMin: minMagnitude,
      colorMax: maxMagnitude,
      removeOutOfBounds: true,
      pixelsPerPoint: 3,
    };
  } else {
    const res = find_min_max_value(gribBytes, key.firstComponent);
    const [minValue, maxValue] = [res.min, res.max];
    return {
      colorMin: minValue,
      colorMax: maxValue,
      removeOutOfBounds: true,
      pixelsPerPoint: 3,
    };
  }
}

interface VectorFieldOverlaySettings {
  arrowType: "PivotTip" | "PivotCenter" | "WindBarb";
  scaleArrow: boolean;
  scaleMax: number | null;
}

function createDefaultVectorFieldOverlaySettings(
  gribBytes: Uint8Array,
  key: GribKey,
): VectorFieldOverlaySettings {
  if (!key.isVectorField) {
    throw new Error("Key must represent a vector field");
  }
  const maxMagnitude = find_min_max_magnitude(
    gribBytes,
    key.firstComponent,
    key.secondComponent!,
  ).max;
  return {
    arrowType: "PivotTip",
    scaleArrow: true,
    scaleMax: maxMagnitude,
  };
}

export {
  createDefaultHeatMapOverlaySettings,
  createDefaultVectorFieldOverlaySettings,
};
