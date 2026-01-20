import type { Map, LatLngBoundsExpression } from "leaflet";
import L, { LatLngBounds } from "leaflet";
import { GribKey } from "./gribKey.js";
import { OverlaySettingsManager } from "./overlaySettings.js";
import { GribViewer } from "../pkg/online_grib_viewer.js";
import { createColorBar, ColorbarControl } from "./colorbarControl.js";

interface HeatMapLayer {
  overlay: L.ImageOverlay;
  parameterKey: GribKey;
  surfaceKey: string;
  time: bigint;
}

interface VectorFieldLayer {
  overlay: L.ImageOverlay;
  parameterKey: GribKey;
  surfaceKey: string;
  time: bigint;
}

class GribOverlayManager {
  gribViewer: GribViewer;
  gribBytes: Uint8Array;
  map: Map;
  overlaySettingsManager: OverlaySettingsManager;
  heatmapLayer: HeatMapLayer | null;
  vectorFieldLayer: VectorFieldLayer | null;
  vectorFieldZoomOverlays: { [zoomLevel: number]: any } | null;
  displayedZoomLevel: number;
  overlayBounds: LatLngBoundsExpression | null;
  colorbarControl: ColorbarControl | null;

  constructor(
    gribBytes: Uint8Array,
    map: Map,
    overlaySettingsManger: OverlaySettingsManager,
  ) {
    this.gribViewer = new GribViewer(gribBytes);
    this.gribBytes = gribBytes;
    this.map = map;
    this.overlaySettingsManager = overlaySettingsManger;
    this.heatmapLayer = null;
    this.vectorFieldLayer = null;
    this.vectorFieldZoomOverlays = null;
    this.displayedZoomLevel = map.getZoom();
    this.overlayBounds = null;
    this.colorbarControl = null;
  }

  free() {
    if (this.gribViewer) {
      this.gribViewer.free(); // Free the Rust/WASM object
    }
    // Clean up other resources
    this.clearHeatMap();
    this.clearVectorField();
  }

  clearHeatMap() {
    if (this.heatmapLayer) {
      this.map.removeLayer(this.heatmapLayer.overlay);
      this.heatmapLayer = null;
    }
    if (this.colorbarControl) {
      this.map.removeControl(this.colorbarControl);
    }
  }

  clearVectorField() {
    if (this.vectorFieldLayer) {
      this.map.removeLayer(this.vectorFieldLayer.overlay);
      this.vectorFieldLayer = null;
    }
  }

  displayVectorField(parameterKey: GribKey, surfaceKey: string, time: bigint) {
    if (
      // if nothing changed, return
      this.vectorFieldLayer &&
      parameterKey === this.vectorFieldLayer.parameterKey &&
      surfaceKey === this.vectorFieldLayer.surfaceKey &&
      time === this.vectorFieldLayer.time
    ) {
      return;
    }

    if (!parameterKey.isVectorField) {
      throw new Error(
        "Only vector fields can be displayed as vector field overlay",
      );
    }

    this.clearVectorField();

    const vectorFieldSettings =
      this.overlaySettingsManager.getVectorFieldSettings(
        this.gribViewer,
        parameterKey,
      );

    // generate wind barb overlay
    let zoomLevel = this.map.getZoom();
    let svgOverlay = this.gribViewer.vector_field_overlay(
      parameterKey.firstComponent,
      parameterKey.secondComponent!,
      surfaceKey,
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
      svgOverlay = this.gribViewer.vector_field_overlay(
        parameterKey.firstComponent,
        parameterKey.secondComponent!,
        surfaceKey,
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

    const overlay = L.imageOverlay(vecFieldUrl, this.overlayBounds, {
      opacity: 1.0,
    }).addTo(this.map);

    this.vectorFieldLayer = {
      overlay: overlay,
      parameterKey: parameterKey,
      surfaceKey: surfaceKey,
      time: time,
    };

    this.displayedZoomLevel = zoomLevel;

    // build a cache of layers at different zoom levels
    this.vectorFieldZoomOverlays = {};
    this.vectorFieldZoomOverlays[zoomLevel] = svgOverlay;

    for (let zl = minZoomLevel; zl <= maxZoomLevel; zl++) {
      if (zl == zoomLevel) {
        continue;
      }
      const svgOverlay = this.gribViewer.vector_field_overlay(
        parameterKey.firstComponent,
        parameterKey.secondComponent!,
        surfaceKey,
        BigInt(time),
        BigInt(zl),
        vectorFieldSettings,
      );
      this.vectorFieldZoomOverlays[zl] = svgOverlay;
    }

    // const data = get_scalar_field(gribBytes, u_key, BigInt(time));
    // markAllPoints(data.lat, data.lon);

    // console.log(arrowZoomLayers);
  }

  displayHeatmap(parameterKey: GribKey, surfaceKey: string, time: bigint) {
    if (
      // if nothing changed, return
      this.heatmapLayer &&
      parameterKey === this.heatmapLayer.parameterKey &&
      surfaceKey === this.heatmapLayer.surfaceKey &&
      time === this.heatmapLayer.time
    ) {
      return;
    }

    this.clearHeatMap();
    let wasmOverlay;
    // let heatmapSettings;

    const heatmapSettings = this.overlaySettingsManager.getHeatmapSettings(
      this.gribViewer,
      parameterKey,
    );

    if (parameterKey.isVectorField) {
      const u_key = parameterKey.firstComponent;
      const v_key = parameterKey.secondComponent!;

      wasmOverlay = this.gribViewer.magnitude_heatmap_overlay(
        u_key,
        v_key,
        surfaceKey,
        time,
        heatmapSettings,
      );
    } else {
      wasmOverlay = this.gribViewer.heatmap_overlay(
        parameterKey.firstComponent,
        surfaceKey,
        time,
        heatmapSettings,
      );
    }

    const canvas = document.createElement("canvas");
    canvas.width = wasmOverlay.widthPx;
    canvas.height = wasmOverlay.heightPx;
    const ctx = canvas.getContext("2d");
    if (!ctx) {
      throw new Error("Could not get 2D context from canvas");
    }
    const imageData = new ImageData(
      new Uint8ClampedArray(wasmOverlay.image),
      wasmOverlay.widthPx,
      wasmOverlay.heightPx,
    );
    ctx.putImageData(imageData, 0, 0);
    const url = canvas.toDataURL();

    const bounds = new LatLngBounds([
      [wasmOverlay.minLat, wasmOverlay.minLon],
      [wasmOverlay.maxLat, wasmOverlay.maxLon],
    ]);

    let overlay = L.imageOverlay(url, bounds, { opacity: 0.4 }).addTo(this.map);

    this.heatmapLayer = {
      overlay: overlay,
      parameterKey: parameterKey,
      surfaceKey: surfaceKey,
      time: time,
    };

    this.overlayBounds = bounds;
    // this.map.fitBounds(bounds);

    const colorbarCanvas = document.createElement("canvas");
    colorbarCanvas.width = 100;
    colorbarCanvas.height = 1;
    const colorbarCtx = colorbarCanvas.getContext("2d");
    if (!colorbarCtx) {
      throw new Error("Could not get 2D context from canvas");
    }
    const colorbarImageData = new ImageData(
      new Uint8ClampedArray(wasmOverlay.colorbarImage),
      100,
      1,
    );
    colorbarCtx.putImageData(colorbarImageData, 0, 0);
    const colorbarUrl = colorbarCanvas.toDataURL();
    this.colorbarControl = createColorBar(
      colorbarUrl,
      wasmOverlay.minValue,
      wasmOverlay.maxValue,
      {
        position: "bottomleft",
      },
    ).addTo(this.map);
  }

  updateZoomLevel() {
    // If there are no cached vector field layers, nothing to do
    if (
      !this.vectorFieldZoomOverlays ||
      Object.keys(this.vectorFieldZoomOverlays).length === 0 ||
      !this.vectorFieldLayer
    ) {
      return;
    }

    // determine zoom level that we want to load
    const maxZoom = Math.max(
      ...Object.keys(this.vectorFieldZoomOverlays).map((zl) => Number(zl)),
    );
    const minZoom = Math.min(
      ...Object.keys(this.vectorFieldZoomOverlays).map((zl) => Number(zl)),
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

    // Add the appropriate layer for the current zoom level
    if (this.vectorFieldZoomOverlays[newZoom]) {
      const svgOverlay = this.vectorFieldZoomOverlays[newZoom];
      const svgBlob = new Blob([svgOverlay.svgString], {
        type: "image/svg+xml;charset=utf-8",
      });
      const url = URL.createObjectURL(svgBlob);

      const bounds = new LatLngBounds([
        [svgOverlay.minLat, svgOverlay.minLon],
        [svgOverlay.maxLat, svgOverlay.maxLon],
      ]);

      // Clear the current vector field layer
      this.map.removeLayer(this.vectorFieldLayer.overlay);

      this.vectorFieldLayer.overlay = L.imageOverlay(url, bounds, {
        opacity: 1.0,
      }).addTo(this.map);
    }
  }
}

export default GribOverlayManager;
