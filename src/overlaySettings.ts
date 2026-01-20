import { GribViewer } from "../pkg/online_grib_viewer.js";
type GribKey = import("./gribKey.js").GribKey;

interface HeatMapOverlaySettings {
  colorMin: Number | null;
  colorMax: Number | null;
  removeOutOfBounds: boolean;
  pixelsPerPoint: number;
}

interface VectorFieldOverlaySettings {
  arrowType: "PivotTip" | "PivotCenter" | "WindBarb";
  scaleArrow: boolean;
  scaleMax: number | null;
}

// settings should be of type Partial<HeatMapOverlaySettings | VectorFieldOverlaySettings>, with one
// exception: colorMin, colorMax and scaleMax can also have the value "FileBased" or "MessageBased"
// UnvalidatedSetting represents this.
interface UnValidatedSetting {
  colorMin?: Number | null | "FileBased" | "MessageBased";
  colorMax?: Number | null | "FileBased" | "MessageBased";
  removeOutOfBounds?: boolean;
  pixelsPerPoint?: number;
  arrowType?: "PivotTip" | "PivotCenter" | "WindBarb";
  scaleArrow?: boolean;
  scaleMax?: number | null | "FileBased" | "MessageBased";
}

class OverlaySettingsManager {
  settings: { [key: string]: UnValidatedSetting };
  gribBytes: Uint8Array;

  constructor(settings: any, gribBytes: Uint8Array) {
    this.settings = settings;
    this.gribBytes = gribBytes;
  }

  // Checks if settings contains the key, and processes it into valid HeatMapOverlaySettings instance
  //
  // It returns a valid HeatMapOverlaySettings instance and updates this.settings with this valid instance,
  // so that FileBased min and max values do not have to be recalculated
  getHeatmapSettings(
    gribViewer: Readonly<GribViewer>,
    key: GribKey,
  ): HeatMapOverlaySettings {
    let min_max;
    let result;

    // see if settings for this key are already available in settings
    if (this.settings.hasOwnProperty(key.toString())) {
      result = this.settings[key.toString()];
    } else {
      result = {};
    }

    // ==== colorMin ====
    if (result.hasOwnProperty("colorMin")) {
      if (result["colorMin"] === "FileBased") {
        // FileBased: find minimum value of variable in whole file
        if (key.isVectorField) {
          min_max = gribViewer.find_min_max_magnitude(
            key.firstComponent,
            key.secondComponent!,
          );
        } else {
          min_max = gribViewer.find_min_max_value(key.firstComponent);
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
      min_max = gribViewer.find_min_max_value(key.firstComponent);
      result["colorMin"] = min_max.min;
    }

    // ==== colorMax ====
    if (result.hasOwnProperty("colorMax")) {
      if (result["colorMax"] === "FileBased") {
        // FileBased: find maximum value of variable in whole file
        if (key.isVectorField) {
          min_max ??= gribViewer.find_min_max_magnitude(
            key.firstComponent,
            key.secondComponent!,
          );
        } else {
          min_max ??= gribViewer.find_min_max_value(key.firstComponent);
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
      min_max ??= gribViewer.find_min_max_value(key.firstComponent);
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

    // update the settings with the validated settings
    this.settings[key.toString()] = {
      ...this.settings[key.toString()],
      ...result,
    };

    return result as HeatMapOverlaySettings;
  }

  // Checks if this.settings contains the key, and processes it into valid HeatMapOverlaySettings instance
  //
  // //
  // It returns a valid VectorFieldOverlaySettings instance and updates this.settings with this valid instance,
  // so that FileBased min and max values do not have to be recalculated
  getVectorFieldSettings(
    gribViewer: Readonly<GribViewer>,
    key: GribKey,
  ): VectorFieldOverlaySettings {
    if (!key.isVectorField) {
      throw new Error("Key is not a vector field");
    }
    let min_max;
    let result;
    if (this.settings.hasOwnProperty(key.toString())) {
      result = this.settings[key.toString()];
    } else {
      result = {};
    }

    // ==== scaleMax ====
    if (result.hasOwnProperty("scaleMax")) {
      if (result["scaleMax"] === "FileBased") {
        // FileBased: find minimum value of variable in whole file
        min_max = gribViewer.find_min_max_magnitude(
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
      min_max = gribViewer.find_min_max_magnitude(
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

    // update the settings with the validated settings
    this.settings[key.toString()] = {
      ...this.settings[key.toString()],
      ...result,
    };

    return result as VectorFieldOverlaySettings;
  }
}

async function loadDefaultSettings(
  gribBytes: Uint8Array,
): Promise<OverlaySettingsManager> {
  try {
    const response = await fetch("/settings/overlaySettings.json");
    if (!response.ok) {
      throw new Error(
        `HTTP error fetching overlaySettings.json, status: ${response.status}`,
      );
    }
    return new OverlaySettingsManager(await response.json(), gribBytes);
  } catch (error) {
    console.error("Error fetching or parsing JSON:", error);
    throw error;
  }
}

export { loadDefaultSettings, OverlaySettingsManager };
