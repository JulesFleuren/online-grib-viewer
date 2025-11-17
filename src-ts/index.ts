import L from "leaflet";
import init, {
  get_available_parameters,
  get_available_timestamps,
  query_grib_message_at_point,
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

// certain parameter pairs are known to be vector fields
const PARAMETER_PAIRS = {
  Wind: { u: "grib2_0_2_2", v: "grib2_0_2_3" },
  Current: { u: "grib2_10_1_2", v: "grib2_10_1_3" },
} as const;

let map: L.Map | null = null;
let gribOverlayManager: GribOverlayManager | null = null;
let selectedTime: bigint = 0n;
let settings: OverlaySettingsManager | null = null;

async function showParameterSelect(file: File) {
  const arrayBuffer = await file.arrayBuffer();
  const gribBytes = new Uint8Array(arrayBuffer);

  if (gribOverlayManager) {
    gribOverlayManager.clearHeatMap();
    gribOverlayManager.clearVectorField();
  }

  settings = await loadDefaultSettings(gribBytes);
  gribOverlayManager = new GribOverlayManager(gribBytes, map!, settings);

  const parameters = get_available_parameters(gribBytes);

  // console.log('Available parameters:', parameters);

  const vectorFieldSelect = document.getElementById(
    "vectorFieldParameterSelect",
  ) as HTMLSelectElement;
  const heatMapSelect = document.getElementById(
    "heatMapParameterSelect",
  ) as HTMLSelectElement;

  heatMapSelect.innerHTML = "";
  vectorFieldSelect.innerHTML = "";

  // add option to plot no vector field
  {
    const emptyOption = document.createElement("option");
    emptyOption.value = "None";
    emptyOption.textContent = "None";
    vectorFieldSelect.appendChild(emptyOption);
  }
  // Check if any parameter pairs are available for vector field display and add an option if so
  Object.keys(PARAMETER_PAIRS).forEach((pairName) => {
    const pair = PARAMETER_PAIRS[pairName as keyof typeof PARAMETER_PAIRS];
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
    heatMapSelect.appendChild(emptyOption);
  }
  // add option to plot magnitude of vector field
  const option = document.createElement("option");
  option.value = "magnitudeVectorField";
  option.id = "magnitudeVectorFieldOption";
  option.textContent = "Magnitude of Vector Field";
  if (vectorFieldSelect.options.length == 0) {
    option.disabled = true;
  }
  heatMapSelect.appendChild(option);

  // add all available parameters as options
  parameters.forEach((p) => {
    const option = document.createElement("option");
    option.value = p.key;
    option.textContent = p.name;
    heatMapSelect.appendChild(option);
  });

  // make sure parameterField is no longer hidden
  (document.getElementById("parameterField") as HTMLElement).style.display = "";

  // automatically select the first parameter and show the timesteps
  if (vectorFieldSelect.options.length > 1) {
    // skip the first because that is the empty option
    vectorFieldSelect.value = vectorFieldSelect.options[1].value;
    heatMapSelect.value = heatMapSelect.options[1].value;
    showTimeSelect(new GribKey(vectorFieldSelect.options[1].value));
  } else if (heatMapSelect.options.length > 2) {
    // skip the first because that is the empty option, skip the second, because that is the
    // magnitudeVectorField option, which should be disabled when this else clause is reached
    showTimeSelect(new GribKey(heatMapSelect.options[2].value));
  }
}

function showTimeSelect(selectedParameter: GribKey) {
  if (!gribOverlayManager) {
    throw new Error("gribOverlayManager is null");
  }

  var availableTimes: bigint[];
  if (selectedParameter.isVectorField) {
    // Vector field
    var timesU = get_available_timestamps(
      gribOverlayManager.gribBytes,
      selectedParameter.firstComponent,
    );
    var timesV = get_available_timestamps(
      gribOverlayManager.gribBytes,
      selectedParameter.secondComponent!,
    );
    // find intersection of timesU and timesV
    availableTimes = timesU.filter((t) => timesV.includes(t));
  } else {
    // Scalar field
    availableTimes = get_available_timestamps(
      gribOverlayManager.gribBytes,
      selectedParameter.firstComponent,
    );
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
    updateDisplayedParameters();
  }
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
  const heatMapSelect = document.getElementById(
    "heatMapParameterSelect",
  ) as HTMLSelectElement;

  const selectedVectorFieldParameter = vectorFieldSelect.value;
  const selectedHeatMapParameter = heatMapSelect.value;

  if (
    selectedHeatMapParameter != "None" &&
    selectedHeatMapParameter != "magnitudeVectorField"
  ) {
    // display parameter as heatmap
    const heatmapKey = new GribKey(selectedHeatMapParameter);
    gribOverlayManager?.displayHeatmap(heatmapKey, selectedTime);
  } else if (
    selectedHeatMapParameter == "magnitudeVectorField" &&
    selectedVectorFieldParameter != "None"
  ) {
    // display magnitude of vectorfield as heatmap
    const vectorKey = new GribKey(selectedVectorFieldParameter);
    gribOverlayManager?.displayHeatmap(vectorKey, selectedTime);
  } else if (selectedHeatMapParameter == "None") {
    // display no heatmap
    gribOverlayManager?.clearHeatMap();
  }

  if (selectedVectorFieldParameter != "None") {
    const vectorKey = new GribKey(selectedVectorFieldParameter);
    gribOverlayManager?.displayVectorField(vectorKey, selectedTime);
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
  const heatMapSelect = document.getElementById(
    "heatMapParameterSelect",
  ) as HTMLSelectElement;

  const selectedVectorFieldParameter = vectorFieldSelect.value;
  const selectedHeatMapParameter = heatMapSelect.value;

  let lat_out, lon_out;

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
    if (u_key && v_key) {
      const u_data = query_grib_message_at_point(
        gribOverlayManager.gribBytes,
        u_key,
        BigInt(selectedTime),
        lat,
        lon,
      );
      const v_data = query_grib_message_at_point(
        gribOverlayManager.gribBytes,
        v_key,
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
  }

  // add data on selected heat map to popupContent
  if (
    selectedHeatMapParameter != "None" &&
    selectedHeatMapParameter != "magnitudeVectorField"
  ) {
    const data = query_grib_message_at_point(
      gribOverlayManager.gribBytes,
      selectedHeatMapParameter,
      BigInt(selectedTime),
      lat,
      lon,
    );

    const parameterName =
      heatMapSelect.options[heatMapSelect.selectedIndex].textContent;

    popupContent += `${parameterName}: ${data.value.toFixed(2)}<br>`;
    lat_out = data.lat;
    lon_out = data.lon;
  }

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
  L.tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", {
    attribution:
      '&copy <a href="https://openstreetmap.org/copyright">OpenStreetMap</a> contributors',
  }).addTo(map);

  // ===== file input event listener =====
  const fileInput = document.getElementById("fileInput");

  if (!(fileInput instanceof HTMLInputElement)) {
    throw new Error("Expected #fileInput to be an <input type='file'> element");
  }

  fileInput.addEventListener("change", async () => {
    const file = fileInput.files?.[0];
    if (!file) return;

    await showParameterSelect(file);
    if (map && gribOverlayManager && gribOverlayManager.overlayBounds) {
      map.fitBounds(gribOverlayManager.overlayBounds);
    }
  });

  // ===== parameter selection fields event listeners =====
  const heatMapSelect = document.getElementById(
    "heatMapParameterSelect",
  ) as HTMLSelectElement;
  const vectorFieldSelect = document.getElementById(
    "vectorFieldParameterSelect",
  ) as HTMLSelectElement;

  heatMapSelect.addEventListener("change", () => {
    const selectedHMParameter = heatMapSelect.value;
    const selectedVFParameter = vectorFieldSelect.value;

    if (
      selectedVFParameter === "None" &&
      selectedHMParameter !== "magnitudeVectorField" &&
      selectedHMParameter !== "None"
    ) {
      showTimeSelect(new GribKey(selectedHMParameter));
    } else if (selectedVFParameter !== "None") {
      updateDisplayedParameters();
    } else if (
      selectedVFParameter === "None" &&
      selectedHMParameter === "None"
    ) {
      if (gribOverlayManager) {
        gribOverlayManager.clearHeatMap();
        gribOverlayManager.clearVectorField();
      }
    }
  });

  vectorFieldSelect.addEventListener("change", () => {
    const selectedVF = vectorFieldSelect.value;
    let selectedHM = heatMapSelect.value;

    const magnitudeOption = document.getElementById(
      "magnitudeVectorFieldOption",
    ) as HTMLOptionElement;

    // if no vectorfield is selected, disable the heatmap option to display magnitude of vector
    // field, and set the heatMapSelect to None if it was set to magnitudeVectorField
    const vfSelected = selectedVF !== "None";
    magnitudeOption.disabled = !vfSelected;

    if (!vfSelected && selectedHM === "magnitudeVectorField") {
      heatMapSelect.value = "None";
      selectedHM = "None";
    }

    if (!vfSelected && selectedHM === "None") {
      if (gribOverlayManager) {
        gribOverlayManager.clearHeatMap();
        gribOverlayManager.clearVectorField();
      }
      return;
    }

    // Show time selector based on current active parameter
    showTimeSelect(
      vfSelected ? new GribKey(selectedVF) : new GribKey(selectedHM),
    );
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
