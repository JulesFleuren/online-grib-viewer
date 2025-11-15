import type { Map, LatLngBoundsExpression } from "leaflet";
import L, { LatLngBounds } from "leaflet";
import type { GribKey } from "./gribKey";
import {
  createDefaultHeatMapOverlaySettings,
  createDefaultVectorFieldOverlaySettings,
} from "./overlaySettings.js";
import {
  vector_field_overlay,
  heatmap_overlay,
  magnitude_heatmap_overlay,
  find_min_max_value,
  find_min_max_magnitude,
} from "../pkg/online_grib_viewer.js";

class GribOverlay {
  gribBytes: Uint8Array;
  map: Map;
  heatmapLayer: L.ImageOverlay | null;
  vectorFieldLayer: L.ImageOverlay | null;
  vectorFieldZoomLayers: { [zoomLevel: number]: any } | null;
  displayedZoomLevel: number;
  overlayBounds: LatLngBoundsExpression | null;

  constructor(gribBytes: Uint8Array, map: Map) {
    this.gribBytes = gribBytes;
    this.map = map;
    this.heatmapLayer = null;
    this.vectorFieldLayer = null;
    this.vectorFieldZoomLayers = null;
    this.displayedZoomLevel = map.getZoom();
    this.overlayBounds = null;
  }

  clearHeatMap() {
    if (this.heatmapLayer) {
      this.map.removeLayer(this.heatmapLayer);
      this.heatmapLayer = null;
    }
  }

  clearVectorField() {
    if (this.vectorFieldLayer) {
      this.map.removeLayer(this.vectorFieldLayer);
      this.vectorFieldLayer = null;
    }
  }

  displayVectorField(key: GribKey, time: bigint) {
    if (!key.isVectorField) {
      throw new Error(
        "Only vector fields can be displayed as vector field overlay",
      );
    }

    this.clearVectorField();

    let vectorFieldSettings = createDefaultVectorFieldOverlaySettings(
      this.gribBytes,
      key,
    );
    // let vectorFieldSettings;
    //   if (`vector:${u_key},${v_key}` in settings) {
    //     vectorFieldSettings = settings[`vector:${u_key},${v_key}`];
    //   } else {
    //     // By default, colorMin and colorMax are set to the min and max occuring value in the whole file, to ensure that
    //     // it is easy to compare two time stamps
    //     let res = find_min_max_magnitude(gribBytes, u_key, v_key);
    //     vectorFieldSettings = { scaleMax: res.max, scaleArrow: true };
    //     settings[`vector:${u_key},${v_key}`] = vectorFieldSettings;
    //   }

    // generate wind barb overlay
    let zoomLevel = this.map.getZoom();
    console.log(vectorFieldSettings);
    let svgOverlay = vector_field_overlay(
      this.gribBytes,
      key.firstComponent,
      key.secondComponent!,
      time,
      BigInt(zoomLevel),
      vectorFieldSettings,
    );

    // maxZoomLevel is the highest zoomLevel for which an svgOverlay is generated, for all higher zoomLevels the
    // svgOverlay of maxZoomLevel is used. maxZoomLevel is the zoomLevel at which all vectors are rendered.
    const maxZoomLevel = svgOverlay.maxZoomLevel;
    zoomLevel = Math.min(zoomLevel, Number(maxZoomLevel));

    // minZoomLevel is the lowest zoomLevel for which an svgOverlay is generated, for all lower zoomLevels the
    // svgOverlay of minZoomLevel is used. minZoomLevel is the zoomLevel at the whole overlay is visible without panning.
    this.overlayBounds = [
      [svgOverlay.minLat, svgOverlay.minLon],
      [svgOverlay.maxLat, svgOverlay.maxLon],
    ];
    const minZoomLevel = this.map.getBoundsZoom(this.overlayBounds);

    if (zoomLevel < minZoomLevel) {
      zoomLevel = minZoomLevel;
      svgOverlay = vector_field_overlay(
        this.gribBytes,
        key.firstComponent,
        key.secondComponent!,
        time,
        BigInt(zoomLevel),
        vectorFieldSettings,
      );
      // svgOverlay.maxZoomLevel, svgOverlay.minLat, ..., svgOverlay.maxLon are independent of zoomLevel
    }
    // TODO: minZoomLevel can only be determined from overlayBounds with leaflet method this.map.getBounds, which means that
    // the vector_field_overlay has to be redrawn when initially zoomLevel < minZoomLevel. Can we avoid this?

    // Now display the wind barbs
    const svgBlob = new Blob([svgOverlay.svgString], {
      type: "image/svg+xml;charset=utf-8",
    });
    const vecFieldUrl = URL.createObjectURL(svgBlob);

    this.vectorFieldLayer = L.imageOverlay(vecFieldUrl, this.overlayBounds, {
      opacity: 1.0,
    }).addTo(this.map);

    this.displayedZoomLevel = zoomLevel;

    // build a cache of layers at different zoom levels
    this.vectorFieldZoomLayers = {};
    this.vectorFieldZoomLayers[zoomLevel] = svgOverlay;

    for (let zl = minZoomLevel; zl <= maxZoomLevel; zl++) {
      if (zl == zoomLevel) {
        continue;
      }
      const svgOverlay = vector_field_overlay(
        this.gribBytes,
        key.firstComponent,
        key.secondComponent!,
        BigInt(time),
        BigInt(zl),
        vectorFieldSettings,
      );
      this.vectorFieldZoomLayers[zl] = svgOverlay;
    }

    // const data = get_scalar_field(gribBytes, u_key, BigInt(time));
    // markAllPoints(data.lat, data.lon);

    // console.log(arrowZoomLayers);
  }

  displayHeatmap(key: GribKey, time: bigint) {
    this.clearHeatMap();
    let imageOverlay;
    // let heatmapSettings;

    if (key.isVectorField) {
      // const selectedVectorFieldParameter = document.getElementById(
      //     "vectorFieldParameterSelect",
      // ).value;
      // if (selectedVectorFieldParameter == "None") {
      //     return;
      // }
      const u_key = key.firstComponent;
      const v_key = key.secondComponent!;

      const heatmapSettings = createDefaultHeatMapOverlaySettings(
        this.gribBytes,
        key,
      );
      // if (selectedVectorFieldParameter in settings) {
      //     heatmapSettings = settings[selectedVectorFieldParameter];
      // } else {
      //     // By default, colorMin and colorMax are set to the min and max occuring value in the whole file, to ensure that
      //     // it is easy to compare two time stamps
      //     let res = find_min_max_magnitude(gribBytes, u_key, v_key);
      //     heatmapSettings = {
      //         colorMin: res.min,
      //         colorMax: res.max,
      //         // scaleMax: res.max,
      //         scaleMax: 1,
      //         scaleArrow: true,
      //     };
      //     settings[selectedVectorFieldParameter] = heatmapSettings;
      // }

      imageOverlay = magnitude_heatmap_overlay(
        this.gribBytes,
        u_key,
        v_key,
        time,
        heatmapSettings,
      );
    } else {
      const heatmapSettings = createDefaultHeatMapOverlaySettings(
        this.gribBytes,
        key,
      );
      // if (key in settings) {
      //     heatmapSettings = settings[key];
      // } else {
      //     // By default, colorMin and colorMax are set to the min and max occuring value in the whole file, to ensure that
      //     // it is easy to compare two time stamps
      //     let res = find_min_max_value(gribBytes, key);
      //     heatmapSettings = { colorMin: res.min, colorMax: res.max };
      //     settings[key] = heatmapSettings;
      // }

      imageOverlay = heatmap_overlay(
        this.gribBytes,
        key.firstComponent,
        time,
        heatmapSettings,
      );
    }

    const canvas = document.createElement("canvas");
    canvas.width = imageOverlay.widthPx;
    canvas.height = imageOverlay.heightPx;
    const ctx = canvas.getContext("2d");
    if (!ctx) {
      throw new Error("Could not get 2D context from canvas");
    }
    const imageData = new ImageData(
      new Uint8ClampedArray(imageOverlay.image),
      imageOverlay.widthPx,
      imageOverlay.heightPx,
    );
    ctx.putImageData(imageData, 0, 0);
    const url = canvas.toDataURL();

    const bounds = new LatLngBounds([
      [imageOverlay.minLat, imageOverlay.minLon],
      [imageOverlay.maxLat, imageOverlay.maxLon],
    ]);

    this.heatmapLayer = L.imageOverlay(url, bounds, { opacity: 0.4 }).addTo(
      this.map,
    );
    this.overlayBounds = bounds;
    // this.map.fitBounds(bounds);
  }

  updateZoomLevel() {
    // If there are no cached vector field layers, nothing to do
    if (
      !this.vectorFieldZoomLayers ||
      Object.keys(this.vectorFieldZoomLayers).length === 0
    ) {
      return;
    }

    // determine zoom level that we want to load
    const maxZoom = Math.max(
      ...Object.keys(this.vectorFieldZoomLayers).map((zl) => Number(zl)),
    );
    const minZoom = Math.min(
      ...Object.keys(this.vectorFieldZoomLayers).map((zl) => Number(zl)),
    );

    let newZoom = this.map.getZoom();
    if (newZoom > maxZoom) {
      if (this.displayedZoomLevel === maxZoom) {
        return;
      } else {
        newZoom = maxZoom;
      }
    }
    if (newZoom < minZoom) {
      if (this.displayedZoomLevel === minZoom) {
        return;
      } else {
        newZoom = minZoom;
      }
    }

    // Clear the current vector field layer
    this.clearVectorField();

    // Add the appropriate layer for the current zoom level
    if (this.vectorFieldZoomLayers[newZoom]) {
      const svgOverlay = this.vectorFieldZoomLayers[newZoom];
      const svgBlob = new Blob([svgOverlay.svgString], {
        type: "image/svg+xml;charset=utf-8",
      });
      const url = URL.createObjectURL(svgBlob);

      const bounds = new LatLngBounds([
        [svgOverlay.minLat, svgOverlay.minLon],
        [svgOverlay.maxLat, svgOverlay.maxLon],
      ]);

      this.vectorFieldLayer = L.imageOverlay(url, bounds, {
        opacity: 1.0,
      }).addTo(this.map);
    }
  }
}

export default GribOverlay;
