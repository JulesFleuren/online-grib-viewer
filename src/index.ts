import L from "leaflet";
import "leaflet.fullscreen";
import { leafletLayer } from "protomaps-leaflet";
import init, {
  get_available_parameters,
  GribViewer,
  get_available_surfaces,
  get_available_timestamps,
  query_grib_message_at_point,
  get_message_info,
  get_message_dump,
} from "../pkg/online_grib_viewer.js";
import GribOverlayManager from "./overlays.js";
import { GribKey } from "./gribKey.js";
import {
  findNextOrEqualTimestamp,
  findNextTimestamp,
  findPreviousTimestamp,
} from "./timeUtils.js";
import {
  loadDefaultSettings,
  OverlaySettingsManager,
} from "./overlaySettings.js";

let map: L.Map | null = null;
let gribOverlayManager: GribOverlayManager | null = null;
let selectedTime: bigint = 0n;
let overlaySettings: OverlaySettingsManager | null = null;
let vectorPairs: { [key: string]: { [key: string]: string } } | null = null;

async function loadPairs() {
  try {
    const response = await fetch("/settings/vectorPairs.json");
    if (!response.ok) {
      throw new Error(
        `HTTP error fetching vectorPairs.json, status: ${response.status}`,
      );
    }
    return response.json();
  } catch (error) {
    console.error("Error fetching or parsing JSON:", error);
    throw error;
  }
}

async function loadFile(file: File) {
  const arrayBuffer = await file.arrayBuffer();
  const gribBytes = new Uint8Array(arrayBuffer);

  if (gribOverlayManager) {
    gribOverlayManager.clearHeatMap();
    gribOverlayManager.clearVectorField();
  }

  overlaySettings = await loadDefaultSettings(gribBytes);
  vectorPairs = await loadPairs();
  gribOverlayManager = new GribOverlayManager(gribBytes, map!, overlaySettings);

  const gribViewer = new GribViewer(gribBytes);

  const parameters = gribViewer.get_available_parameters();

  const vectorFieldSelect = document.getElementById(
    "vectorFieldParameterSelect",
  ) as HTMLSelectElement;
  const heatmapSelect = document.getElementById(
    "heatmapParameterSelect",
  ) as HTMLSelectElement;

  heatmapSelect.innerHTML = "";
  vectorFieldSelect.innerHTML = "";

  // add option to plot no vector field
  {
    const emptyOption = document.createElement("option");
    emptyOption.value = "None";
    emptyOption.textContent = "None";
    vectorFieldSelect.appendChild(emptyOption);
  }
  // Check if any parameter pairs are available for vector field display and add an option if so
  Object.keys(vectorPairs ?? {}).forEach((pairName) => {
    const pair = vectorPairs![pairName as keyof typeof vectorPairs];
    const hasU = parameters.some((p) => p.key === pair.u);
    const hasV = parameters.some((p) => p.key === pair.v);
    if (hasU && hasV) {
      const vectorFieldOption = document.createElement("option");
      vectorFieldOption.value = `vector:${pair.u},${pair.v}`;
      vectorFieldOption.textContent = `${pairName}`;
      vectorFieldSelect.appendChild(vectorFieldOption);
    }
  });

  // add option to plot no heat map
  {
    const emptyOption = document.createElement("option");
    emptyOption.value = "None";
    emptyOption.textContent = "None";
    heatmapSelect.appendChild(emptyOption);
  }
  // add option to plot magnitude of vector field
  const magVFoption = document.createElement("option");
  magVFoption.value = "magnitudeVectorField";
  magVFoption.id = "magnitudeVectorFieldOption";
  magVFoption.textContent = "Magnitude of Vector Field";
  if (vectorFieldSelect.options.length == 1) {
    magVFoption.disabled = true;
  }
  heatmapSelect.appendChild(magVFoption);

  // add all available parameters as options
  parameters.forEach((p) => {
    const option = document.createElement("option");
    option.value = p.key;
    option.textContent = p.name;
    heatmapSelect.appendChild(option);
  });

  // make sure parameterField is no longer hidden
  (document.getElementById("parameterField") as HTMLElement).style.display = "";

  // automatically select the first parameter and show the timesteps
  if (vectorFieldSelect.options.length > 1) {
    // skip the first because that is the empty option
    vectorFieldSelect.value = vectorFieldSelect.options[1].value;
    heatmapSelect.value = heatmapSelect.options[1].value;
  } else {
    vectorFieldSelect.value = vectorFieldSelect.options[0].value;
    if (heatmapSelect.options.length > 2) {
      // skip the first because that is the empty option, skip the second, because that is the
      // magnitudeVectorField option, which should be disabled when this else clause is reached
      heatmapSelect.value = heatmapSelect.options[2].value;
    } else {
      // No grib messages found
      heatmapSelect.value = heatmapSelect.options[0].value;
    }
  }
  updateVectorFieldSurfaceSelect();
}

function updateVectorFieldSurfaceSelect() {
  if (!gribOverlayManager) {
    throw new Error("gribOverlayManager is null");
  }

  const vectorFieldSelect = document.getElementById(
    "vectorFieldParameterSelect",
  ) as HTMLSelectElement;
  const selectedVFParameter = vectorFieldSelect.value;

  const vectorFieldSurfaceSelect = document.getElementById(
    "vectorFieldSurfaceSelect",
  ) as HTMLSelectElement;

  vectorFieldSurfaceSelect.innerHTML = "";

  if (selectedVFParameter === "None") {
    vectorFieldSurfaceSelect.disabled = true;
    updateHeatmapSurfaceSelect();
    return;
  }

  const selectedParameter = new GribKey(selectedVFParameter);

  var surfacesU = get_available_surfaces(
    gribOverlayManager.gribBytes,
    selectedParameter.firstComponent,
  );

  var surfacesV = get_available_surfaces(
    gribOverlayManager.gribBytes,
    selectedParameter.secondComponent!,
  );

  // find intersection of surfacesU and surfacesV
  const availableSurfaces = surfacesU.filter((u) =>
    surfacesV.some((v) => v.key === u.key),
  );

  if (availableSurfaces.length == 0) {
    throw new Error(
      `No valid fixed surfaces for vector field ${selectedParameter.toString()}`,
    );
  }

  availableSurfaces.forEach((s) => {
    const option = document.createElement("option");
    option.value = s.key;
    option.textContent = s.description;
    vectorFieldSurfaceSelect.appendChild(option);
  });

  // automatically select the first parameter
  vectorFieldSurfaceSelect.value = vectorFieldSurfaceSelect.options[0].value;
  vectorFieldSurfaceSelect.disabled = false;
  updateHeatmapSurfaceSelect();
}

function updateHeatmapSurfaceSelect() {
  if (!gribOverlayManager) {
    throw new Error("gribOverlayManager is null");
  }

  const heatmapSelect = document.getElementById(
    "heatmapParameterSelect",
  ) as HTMLSelectElement;
  const selectedHMParameter = heatmapSelect.value;

  const heatmapSurfaceSelect = document.getElementById(
    "heatmapSurfaceSelect",
  ) as HTMLSelectElement;

  const infoBtn = document.getElementById(
    "messageInfoButton",
  ) as HTMLButtonElement;

  heatmapSurfaceSelect.innerHTML = "";

  if (
    selectedHMParameter === "None" ||
    selectedHMParameter === "magnitudeVectorField"
  ) {
    heatmapSurfaceSelect.disabled = true;
    infoBtn.disabled = true;
    updateTimeSelect();
    return;
  }

  const selectedParameter = new GribKey(selectedHMParameter);

  const availableSurfaces = get_available_surfaces(
    gribOverlayManager.gribBytes,
    selectedParameter.firstComponent,
  );

  if (availableSurfaces.length == 0) {
    throw new Error(
      `No valid fixed surfaces for parameter ${selectedParameter.toString()}`,
    );
  }

  availableSurfaces.forEach((s) => {
    const option = document.createElement("option");
    option.value = s.key;
    option.textContent = s.description;
    heatmapSurfaceSelect.appendChild(option);
  });

  // automatically select the first parameter and show the timesteps
  heatmapSurfaceSelect.value = heatmapSurfaceSelect.options[0].value;
  heatmapSurfaceSelect.disabled = false;
  infoBtn.disabled = false;
  updateTimeSelect();
}

function updateTimeSelect() {
  if (!gribOverlayManager) {
    throw new Error("gribOverlayManager is null");
  }

  const heatmapSelect = document.getElementById(
    "heatmapParameterSelect",
  ) as HTMLSelectElement;
  const vectorFieldSelect = document.getElementById(
    "vectorFieldParameterSelect",
  ) as HTMLSelectElement;

  const selectedHMParameter = heatmapSelect.value;
  const selectedVFParameter = vectorFieldSelect.value;

  const vectorFieldSurfaceSelect = document.getElementById(
    "vectorFieldSurfaceSelect",
  ) as HTMLSelectElement;
  let selectedVFSurface = vectorFieldSurfaceSelect.value ?? null;

  const heatmapSurfaceSelect = document.getElementById(
    "heatmapSurfaceSelect",
  ) as HTMLSelectElement;
  let selectedHMSurface = heatmapSurfaceSelect.value ?? null;

  var availableTimes: bigint[];
  if (selectedVFParameter !== "None") {
    var selectedParameter = new GribKey(selectedVFParameter);
    // Vector field
    var timesU = get_available_timestamps(
      gribOverlayManager.gribBytes,
      selectedParameter.firstComponent,
      selectedVFSurface,
    );
    var timesV = get_available_timestamps(
      gribOverlayManager.gribBytes,
      selectedParameter.secondComponent!,
      selectedVFSurface,
    );
    // find intersection of timesU and timesV
    availableTimes = timesU.filter((t) => timesV.includes(t));
  } else if (selectedHMParameter !== "None") {
    var selectedParameter = new GribKey(selectedHMParameter);
    // Scalar field
    availableTimes = get_available_timestamps(
      gribOverlayManager.gribBytes,
      selectedParameter.firstComponent,
      selectedHMSurface,
    );
  } else {
    availableTimes = [];
  }

  // add avaialble times as options to timestampSelect
  const timestampSelect = document.getElementById(
    "timestampSelect",
  ) as HTMLSelectElement;

  timestampSelect.innerHTML = "";
  availableTimes.forEach((t) => {
    const option = document.createElement("option");
    option.value = String(t);
    option.textContent = new Date(Number(t) * 1000).toString();
    timestampSelect.appendChild(option);
  });

  // ensure that timestampField is no longer hidden
  (document.getElementById("timestampField") as HTMLElement).style.display = "";

  // automatically select timestamp
  if (availableTimes.length > 0) {
    selectedTime = findNextOrEqualTimestamp(
      selectedTime,
      availableTimes.map((t) => BigInt(t)),
    );
    timestampSelect.value = String(selectedTime);
  }
  updateDisplayedParameters();
}

// function displayCanvas(canvas) {
//   // display the canvas on the bottom of the page for debugging
//   document.body.appendChild(canvas);
// }

// function displaySvg(svg) {
//   // display the svg on the bottom of the page for debugging
//   document.body.appendChild(svg.node());
// }

// function markAllPoints(lat, lon) {
//   for (let i = 0; i < lat.length; i++) {
//     if (isNaN(lat[i]) || isNaN(lon[i])) continue;
//     const marker = L.circleMarker([lat[i], lon[i]], { radius: 1 }).addTo(map!);
//   }
// }

function updateDisplayedParameters() {
  if (!selectedTime) {
    throw new Error("selectedTime is null");
  }
  const vectorFieldSelect = document.getElementById(
    "vectorFieldParameterSelect",
  ) as HTMLSelectElement;
  const heatmapSelect = document.getElementById(
    "heatmapParameterSelect",
  ) as HTMLSelectElement;

  const selectedVectorFieldParameter = vectorFieldSelect.value;
  const selectedHeatMapParameter = heatmapSelect.value;

  const vectorFieldSurfaceSelect = document.getElementById(
    "vectorFieldSurfaceSelect",
  ) as HTMLSelectElement;
  const selectedVFSurface = vectorFieldSurfaceSelect.value;

  const heatmapSurfaceSelect = document.getElementById(
    "heatmapSurfaceSelect",
  ) as HTMLSelectElement;
  const selectedHMSurface = heatmapSurfaceSelect.value;

  if (
    selectedHeatMapParameter != "None" &&
    selectedHeatMapParameter != "magnitudeVectorField"
  ) {
    // display parameter as heatmap
    const heatmapKey = new GribKey(selectedHeatMapParameter);
    gribOverlayManager?.displayHeatmap(
      heatmapKey,
      selectedHMSurface,
      selectedTime,
    );
  } else if (
    selectedHeatMapParameter == "magnitudeVectorField" &&
    selectedVectorFieldParameter != "None"
  ) {
    // display magnitude of vectorfield as heatmap
    const vectorKey = new GribKey(selectedVectorFieldParameter);
    gribOverlayManager?.displayHeatmap(
      vectorKey,
      selectedVFSurface,
      selectedTime,
    );
  } else if (selectedHeatMapParameter == "None") {
    // display no heatmap
    gribOverlayManager?.clearHeatMap();
  }

  if (selectedVectorFieldParameter != "None") {
    const vectorKey = new GribKey(selectedVectorFieldParameter);
    gribOverlayManager?.displayVectorField(
      vectorKey,
      selectedVFSurface,
      selectedTime,
    );
  } else {
    gribOverlayManager?.clearVectorField();
  }
}

// Function to update the zoom level display
function updateZoomLevel() {
  if (!map) {
    return;
  }
  const zoomLevelDiv = document.getElementById("zoom-level") as HTMLElement;
  zoomLevelDiv.textContent = `Zoom Level: ${map.getZoom()}`;
  gribOverlayManager?.updateZoomLevel();
}

function popupClosestGridPoint(lat: number, lon: number) {
  if (!map) {
    throw new Error("map is null");
  }

  const vectorFieldSelect = document.getElementById(
    "vectorFieldParameterSelect",
  ) as HTMLSelectElement;
  const heatmapSelect = document.getElementById(
    "heatmapParameterSelect",
  ) as HTMLSelectElement;

  const selectedVectorFieldParameter = vectorFieldSelect.value;
  const selectedHeatMapParameter = heatmapSelect.value;

  const vectorFieldSurfaceSelect = document.getElementById(
    "vectorFieldSurfaceSelect",
  ) as HTMLSelectElement;
  const selectedVFSurface = vectorFieldSurfaceSelect.value;

  const heatmapSurfaceSelect = document.getElementById(
    "heatmapSurfaceSelect",
  ) as HTMLSelectElement;
  const selectedHMSurface = heatmapSurfaceSelect.value;

  let lat_out: number, lon_out: number;

  if (
    !gribOverlayManager ||
    !(selectedVectorFieldParameter || selectedHeatMapParameter) ||
    selectedTime === null
  ) {
    return;
  }

  let popupContent = "";

  // add data on selected vector field to popupContent
  if (selectedVectorFieldParameter != "None") {
    const [u_key, v_key] = selectedVectorFieldParameter
      .split(":")[1]
      .split(",");
    const parameterName =
      vectorFieldSelect.options[vectorFieldSelect.selectedIndex].textContent;
    const u_data = query_grib_message_at_point(
      gribOverlayManager.gribBytes,
      u_key,
      selectedVFSurface,
      BigInt(selectedTime),
      lat,
      lon,
    );
    const v_data = query_grib_message_at_point(
      gribOverlayManager.gribBytes,
      v_key,
      selectedVFSurface,
      BigInt(selectedTime),
      lat,
      lon,
    );

    popupContent +=
      `${parameterName}:<br>` +
      `&emsp;U: ${u_data.value.toFixed(2)}<br>` +
      `&emsp;V: ${v_data.value.toFixed(2)}<br>` +
      `&emsp;Speed: ${Math.sqrt(u_data.value ** 2 + v_data.value ** 2).toFixed(2)}<br>` +
      `&emsp;Direction: ${(90 - (Math.atan2(v_data.value, u_data.value) * 180) / Math.PI).toFixed(2)}°<br>`;
    // TODO: should wind direction be inverted?
    lat_out = u_data.lat;
    lon_out = u_data.lon;
  }

  // add data on selected heat map to popupContent
  if (
    selectedHeatMapParameter != "None" &&
    selectedHeatMapParameter != "magnitudeVectorField"
  ) {
    const data = query_grib_message_at_point(
      gribOverlayManager.gribBytes,
      selectedHeatMapParameter,
      selectedHMSurface,
      BigInt(selectedTime),
      lat,
      lon,
    );

    const parameterName =
      heatmapSelect.options[heatmapSelect.selectedIndex].textContent;

    popupContent += `${parameterName}: ${data.value.toFixed(2)}<br>`;
    lat_out = data.lat;
    lon_out = data.lon;
  }

  lat_out ??= lat;
  lon_out ??= lon;

  popupContent =
    "Closest grid point:<br>" +
    `&emsp;lat: ${lat_out.toFixed(8)}<br>` +
    `&emsp;lon: ${lon_out.toFixed(8)}<br>` +
    popupContent;

  // show popup with queried data
  L.popup()
    .setLatLng({ lat: lat_out, lng: lon_out })
    .setContent(popupContent)
    .openOn(map);
}

init().then(() => {
  // epoch time in seconds instead of miliseconds
  selectedTime = BigInt(Math.floor(Date.now() / 1000));
  map = L.map("map", {
    fullscreenControl: true,
    fullscreenControlOptions: {
      position: "topleft",
    },
  }).setView([0, 0], 2);

  const basemapUrl = "https://api.protomaps.com/tiles/v4/{z}/{x}/{y}.mvt";
  const layer = leafletLayer({
    // @ts-expect-error: some weird error about env not being a recognised property
    url: basemapUrl + `?key=${import.meta.env.VITE_PROTOMAPS_API_KEY}`,
    flavor: "light",
    lang: "en",
    maxDataZoom: 11,
  });
  layer.addTo(map);

  // ===== file input event listener =====
  const fileInput = document.getElementById("fileInput");

  if (!(fileInput instanceof HTMLInputElement)) {
    throw new Error("Expected #fileInput to be an <input type='file'> element");
  }

  fileInput.addEventListener("change", async () => {
    const file = fileInput.files?.[0];
    if (!file) return;

    await loadFile(file);
    if (map && gribOverlayManager && gribOverlayManager.overlayBounds) {
      map.fitBounds(gribOverlayManager.overlayBounds);
    }
  });

  // ===== parameter selection fields event listeners =====
  const heatmapSelect = document.getElementById(
    "heatmapParameterSelect",
  ) as HTMLSelectElement;
  const vectorFieldSelect = document.getElementById(
    "vectorFieldParameterSelect",
  ) as HTMLSelectElement;

  const vectorFieldSurfaceSelect = document.getElementById(
    "vectorFieldSurfaceSelect",
  ) as HTMLSelectElement;

  const heatmapSurfaceSelect = document.getElementById(
    "heatmapSurfaceSelect",
  ) as HTMLSelectElement;

  heatmapSelect.addEventListener("change", () => {
    updateHeatmapSurfaceSelect();
  });

  vectorFieldSelect.addEventListener("change", () => {
    const selectedVF = vectorFieldSelect.value;
    let selectedHM = heatmapSelect.value;

    const magnitudeOption = document.getElementById(
      "magnitudeVectorFieldOption",
    ) as HTMLOptionElement;

    // if no vectorfield is selected, disable the heatmap option to display magnitude of vector
    // field, and set the heatmapSelect to None if it was set to magnitudeVectorField
    const vfSelected = selectedVF !== "None";
    magnitudeOption.disabled = !vfSelected;

    if (!vfSelected && selectedHM === "magnitudeVectorField") {
      heatmapSelect.value = "None";
      updateHeatmapSurfaceSelect();
    }

    updateVectorFieldSurfaceSelect();
  });

  vectorFieldSurfaceSelect.addEventListener("change", () => {
    updateTimeSelect();
  });

  heatmapSurfaceSelect.addEventListener("change", () => {
    updateTimeSelect();
  });

  // ===== time related fields event listeners =====
  const timestampSelect = document.getElementById(
    "timestampSelect",
  ) as HTMLSelectElement;
  const nowBtn = document.getElementById(
    "nowTimestampButton",
  ) as HTMLButtonElement;
  const nextBtn = document.getElementById(
    "nextTimestampButton",
  ) as HTMLButtonElement;
  const prevBtn = document.getElementById(
    "prevTimestampButton",
  ) as HTMLButtonElement;

  const getAvailableTimes = () =>
    Array.from(timestampSelect.options, (o) => BigInt(o.value));

  const updateSelectedTime = (t: bigint) => {
    selectedTime = t;
    timestampSelect.value = String(t);
    updateDisplayedParameters();
  };

  timestampSelect.addEventListener("change", (e) => {
    updateSelectedTime(BigInt((e.target as HTMLSelectElement).value));
  });

  nowBtn.addEventListener("click", () => {
    const times = getAvailableTimes();
    const now = BigInt(Math.floor(Date.now() / 1000));
    updateSelectedTime(findNextOrEqualTimestamp(now, times));
  });

  nextBtn.addEventListener("click", () => {
    updateSelectedTime(findNextTimestamp(selectedTime!, getAvailableTimes()));
  });

  prevBtn.addEventListener("click", () => {
    updateSelectedTime(
      findPreviousTimestamp(selectedTime!, getAvailableTimes()),
    );
  });

  // ===== message info button event listeners =====
  const infoBtn = document.getElementById(
    "messageInfoButton",
  ) as HTMLButtonElement;
  const infoModal = document.getElementById(
    "messageInfoModal",
  ) as HTMLDivElement;
  const modalCloseBtn = document.getElementById(
    "modalCloseButton",
  ) as HTMLButtonElement;
  const dumpBtn = document.getElementById(
    "messageDumpButton",
  ) as HTMLButtonElement;

  infoBtn.addEventListener("click", () => {
    const heatmapSelect = document.getElementById(
      "heatmapParameterSelect",
    ) as HTMLSelectElement;
    const selectedHMParameter = heatmapSelect.value;

    const heatmapSurfaceSelect = document.getElementById(
      "heatmapSurfaceSelect",
    ) as HTMLSelectElement;
    let selectedHMSurface = heatmapSurfaceSelect.value ?? null;

    if (
      !gribOverlayManager ||
      selectedHMParameter == "None" ||
      selectedHMParameter == "magnitudeVectorField"
    ) {
      return;
    }

    const content = get_message_info(
      gribOverlayManager.gribBytes,
      selectedHMParameter,
      selectedHMSurface,
      selectedTime!,
    );

    const messageInfoModalBody = document.getElementById(
      "messageInfoModalBody",
    ) as HTMLElement;
    messageInfoModalBody.textContent = content;
    infoModal.classList.add("is-active");
  });

  modalCloseBtn.addEventListener("click", () => {
    infoModal.classList.remove("is-active");
  });

  dumpBtn.addEventListener("click", () => {
    const heatmapSelect = document.getElementById(
      "heatmapParameterSelect",
    ) as HTMLSelectElement;
    const selectedHMParameter = heatmapSelect.value;

    const heatmapSurfaceSelect = document.getElementById(
      "heatmapSurfaceSelect",
    ) as HTMLSelectElement;
    let selectedHMSurface = heatmapSurfaceSelect.value ?? null;

    if (
      !gribOverlayManager ||
      selectedHMParameter == "None" ||
      selectedHMParameter == "magnitudeVectorField"
    ) {
      return;
    }

    const dump = get_message_dump(
      gribOverlayManager.gribBytes,
      selectedHMParameter,
      selectedHMSurface,
      selectedTime!,
    );

    // Open in new tab
    const blob = new Blob([dump], { type: "text/plain" });
    const url = URL.createObjectURL(blob);

    window.open(url, "_blank");
  });

  // ===== interaction with map =====
  // popup when clicking on map
  map.on("click", function (e) {
    const lat = e.latlng.lat;
    const lon = e.latlng.lng;
    popupClosestGridPoint(lat, lon);
  });

  // Initial update
  updateZoomLevel();

  // Update on zoom end
  map.on("zoomend", updateZoomLevel);
});
