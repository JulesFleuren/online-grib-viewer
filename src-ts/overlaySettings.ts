import {
  find_min_max_value,
  find_min_max_magnitude,
} from "../pkg/online_grib_viewer.js";
type GribKey = import("./gribKey.js").GribKey;

let settings: any = null;

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

async function loadDefaultSettings() {
  try {
    const response = await fetch("/settings.json");
    if (!response.ok) {
      throw new Error(
        `HTTP error fetching settings.json, status: ${response.status}`,
      );
    }
    settings = await response.json();
  } catch (error) {
    console.error("Error fetching or parsing JSON:", error);
    throw error;
  }
}

// Checks if settings contains the key, and processes it into valid HeatMapOverlaySettings instance
function getHeatmapSettings(
  key: GribKey,
  gribBytes: Uint8Array,
): HeatMapOverlaySettings {
  let min_max;
  let result;
  if (settings.hasOwnProperty(key.toString())) {
    result = settings[key.toString()];
  } else {
    result = {};
  }

  // ==== colorMin ====
  if (result.hasOwnProperty("colorMin")) {
    if (result["colorMin"] === "FileBased") {
      // FileBased: find minimum value of variable in whole file
      if (key.isVectorField) {
        min_max = find_min_max_magnitude(
          gribBytes,
          key.firstComponent,
          key.secondComponent!,
        );
      } else {
        min_max = find_min_max_value(gribBytes, key.firstComponent);
      }
      result["colorMin"] = min_max.min;
    } else if (result["colorMin"] === "MessageBased") {
      // MessageBased: set to null (see rust implementation)
      result["colorMin"] = null;
    } else if (
      typeof result["colorMin"] === "number" ||
      result["colorMin"] == null
    ) {
      // number or null: do nothing
    } else {
      // otherwise: raise error
      throw new Error(
        `Invalid value for \'colorMin\' at key: ${key.toString()}`,
      );
    }
  } else {
    // default: same as FileBased
    min_max = find_min_max_value(gribBytes, key.firstComponent);
    result["colorMin"] = min_max.min;
  }

  // ==== colorMax ====
  if (result.hasOwnProperty("colorMax")) {
    if (result["colorMax"] === "FileBased") {
      // FileBased: find maximum value of variable in whole file
      if (key.isVectorField) {
        min_max ??= find_min_max_magnitude(
          gribBytes,
          key.firstComponent,
          key.secondComponent!,
        );
      } else {
        min_max ??= find_min_max_value(gribBytes, key.firstComponent);
      }
      result["colorMax"] = min_max.max;
    } else if (result["colorMax"] === "MessageBased") {
      // MessageBased: set to null (see rust implementation)
      result["colorMax"] = null;
    } else if (
      typeof result["colorMax"] === "number" ||
      result["colorMax"] == null
    ) {
      // number or null: do nothing
    } else {
      // otherwise: raise error
      throw new Error(
        `Invalid value for \'colorMax\' at key: ${key.toString()}`,
      );
    }
  } else {
    // default: same as FileBased
    min_max ??= find_min_max_value(gribBytes, key.firstComponent);
    result["colorMax"] = min_max.max;
  }

  // ==== removeOutOfBounds ====
  if (result.hasOwnProperty("removeOutOfBounds")) {
    if (typeof result["removeOutOfBounds"] === "boolean") {
      // boolean: do nothing
    } else {
      // otherwise: raise error
      throw new Error(
        `Invalid value for \'removeOutOfBounds\' at key: ${key.toString()}`,
      );
    }
  } else {
    // default: false
    result["removeOutOfBounds"] = false;
  }

  // ==== pixelsPerPoint ====
  if (result.hasOwnProperty("pixelsPerPoint")) {
    if (
      typeof result["pixelsPerPoint"] === "number" &&
      Number.isInteger(result["pixelsPerPoint"])
    ) {
      // integer: do nothing
    } else {
      // otherwise: raise error
      throw new Error(
        `Invalid value for \'pixelsPerPoint\' at key: ${key.toString()}`,
      );
    }
  } else {
    // default: 3
    result["pixelsPerPoint"] = 3;
  }

  return result as HeatMapOverlaySettings;
}

// Checks if settings contains the key, and processes it into valid HeatMapOverlaySettings instance
function getVectorFieldSettings(
  key: GribKey,
  gribBytes: Uint8Array,
): VectorFieldOverlaySettings {
  if (!key.isVectorField) {
    throw new Error("Key is not a vector field");
  }
  let min_max;
  let result;
  if (settings.hasOwnProperty(key.toString())) {
    result = settings[key.toString()];
  } else {
    result = {};
  }

  // ==== scaleMax ====
  if (result.hasOwnProperty("scaleMax")) {
    if (result["scaleMax"] === "FileBased") {
      // FileBased: find minimum value of variable in whole file
      min_max = find_min_max_magnitude(
        gribBytes,
        key.firstComponent,
        key.secondComponent!,
      );
      result["scaleMax"] = min_max.max;
    } else if (result["scaleMax"] === "MessageBased") {
      // MessageBased: set to null (see rust implementation)
      result["scaleMax"] = null;
    } else if (
      typeof result["scaleMax"] === "number" ||
      result["scaleMax"] == null
    ) {
      // number or null: do nothing
    } else {
      // otherwise: raise error
      throw new Error(
        `Invalid value for \'scaleMax\' at key: ${key.toString()}`,
      );
    }
  } else {
    // default: same as FileBased
    min_max = find_min_max_magnitude(
      gribBytes,
      key.firstComponent,
      key.secondComponent!,
    );
    result["scaleMax"] = min_max.max;
  }

  // ==== scaleArrow ====
  if (result.hasOwnProperty("scaleArrow")) {
    if (typeof result["scaleArrow"] === "boolean") {
      // boolean: do nothing
    } else {
      // otherwise: raise error
      throw new Error(
        `Invalid value for \'scaleArrow\' at key: ${key.toString()}`,
      );
    }
  } else {
    // default: true
    result["scaleArrow"] = true;
  }

  // ==== arrowType ====
  if (result.hasOwnProperty("arrowType")) {
    if (
      result["arrowType"] === "PivotTip" ||
      result["arrowType"] === "PivotCenter" ||
      result["arrowType"] === "WindBarb"
    ) {
      // valid string: do nothing
    } else {
      // otherwise: raise error
      throw new Error(
        `Invalid value for \'arrowType\' at key: ${key.toString()}`,
      );
    }
  } else {
    // default: PivotTip
    result["arrowType"] = "PivotTip";
  }

  return result as VectorFieldOverlaySettings;
}

export {
  loadDefaultSettings,
  getHeatmapSettings,
  getVectorFieldSettings,
  createDefaultHeatMapOverlaySettings,
  createDefaultVectorFieldOverlaySettings,
};
