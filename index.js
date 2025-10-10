import init, { get_available_parameters,
  get_available_timestamps,
  query_grib_message_at_point,
  vector_field_overlay,
  heatmap_overlay,
  magnitude_heatmap_overlay,
  get_scalar_field} from './pkg/online_grib_viewer.js';

// certain parameter pairs are known to be vector fields
const PARAMETER_PAIRS = {
  'Wind': { u: "grib2_0_2_2", v: "grib2_0_2_3" },
  'Current': { u: "grib2_10_1_2", v: "grib2_10_1_3" },
};

const ArrowType = {
  PIVOT_TIP: "PivotTip",
  PIVOT_CENTER: "PivotCenter",
  WIND_BARB: "WindBarb",
}

let map;
let heatLayer;
let arrowLayers = [];
let arrowZoomLayers = {};
let gribBytes = null;
let parameters = [];
let selectedHeatMapParameter = null;
let selectedVectorFieldParameter = null;
let selectedTime = null;
let overlayBounds = null;

function clearHeatMap() {
  if (heatLayer) {
    map.removeLayer(heatLayer);
    heatLayer = null;
  }
}

function clearVectorField() {
  arrowLayers.forEach(layer => map.removeLayer(layer));
  arrowLayers = [];
}

async function showParameterSelect(file) {
  const arrayBuffer = await file.arrayBuffer();
  gribBytes = new Uint8Array(arrayBuffer);

  parameters = get_available_parameters(gribBytes)

  // console.log('Available parameters:', parameters);

  const vectorFieldSelect = document.getElementById('vectorFieldParameterSelect');
  vectorFieldSelect.innerHTML = '';
  // add option to plot no vector field
  {
    const emptyOption = document.createElement('option');
    emptyOption.value = "None";
    emptyOption.textContent = "None";
    vectorFieldSelect.appendChild(emptyOption);
  }
  // Check if any parameter pairs are available for vector field display and add an option if so
  Object.keys(PARAMETER_PAIRS).forEach(pairName => {
    const pair = PARAMETER_PAIRS[pairName];
    const hasU = parameters.some(p => p.key === pair.u);
    const hasV = parameters.some(p => p.key === pair.v);
    if (hasU && hasV) {
      const vectorFieldOption = document.createElement('option');
      vectorFieldOption.value = `vector:${pair.u},${pair.v}`;
      vectorFieldOption.textContent = `${pairName}`;
      vectorFieldSelect.appendChild(vectorFieldOption);
    }
  });

  const heatMapSelect = document.getElementById('heatMapParameterSelect');
  heatMapSelect.innerHTML = '';

  // add option to plot no heat map
  {
    const emptyOption = document.createElement('option');
    emptyOption.value = "None";
    emptyOption.textContent = "None";
    heatMapSelect.appendChild(emptyOption);
  }
  // add option to plot magnitude of vector field
  const option = document.createElement('option');
  option.value = "magnitudeVectorField";
  option.id = "magnitudeVectorFieldOption";
  option.textContent = "Magnitude of Vector Field";
  if (vectorFieldSelect.options.length == 0) {
    option.disabled = true;
  }
  heatMapSelect.appendChild(option);

  // add all available parameters as options
  parameters.forEach(p => {
    const option = document.createElement('option');
    option.value = p.key;
    option.textContent = p.name;
    heatMapSelect.appendChild(option);
  });

  document.getElementById('parameterField').style.display = '';

  // automatically select the first parameter and show the timesteps
  if (vectorFieldSelect.options.length > 1) {
    // skip the first because that is the empty option
    vectorFieldSelect.value = vectorFieldSelect.options[1].value;
    heatMapSelect.value = heatMapSelect.options[1].value;
    showTimeSelect(vectorFieldSelect.options[1].value);
  } else if (heatMapSelect.options.length > 2) {
    // skip the first because that is the empty option, skip the second, because that is the
    // magnitudeVectorField option, which should be disabled when this else clause is reached
    showTimeSelect(heatMapSelect.options[2].value);
  }
}

function showTimeSelect(selectedParameter) {
  if (selectedParameter.startsWith('vector')) {
    // Vector field
    const [u_key, v_key] = selectedParameter.split(":")[1].split(',');
    if (u_key && v_key) {
      var timesU = get_available_timestamps(gribBytes, u_key);
      var timesV = get_available_timestamps(gribBytes, v_key);
      // find intersection of timesU and timesV
      var times = timesU.filter(t => timesV.includes(t));
    }
  } else {
    // Scalar field
    const key = selectedParameter;
    if (key) {
      var times = get_available_timestamps(gribBytes, key);
    }
  }
  const select = document.getElementById('timestampSelect');
  select.innerHTML = '';
  times.forEach(t => {
    const option = document.createElement('option');
    option.value = t;
    option.textContent = new Date(Number(t) * 1000).toString();
    select.appendChild(option);
  });
  document.getElementById('timestampField').style.display = '';

  // automatically select timestamp
  if (times.length > 0) {
    selectedTime = findNextOrEqualTimestamp(selectedTime, times.map(t => Number(t)));
    select.value = selectedTime;
    displayParameters(selectedTime);
  }
}



function displayCanvas(canvas) {
  // display the canvas on the bottom of the page for debugging
  document.body.appendChild(canvas);
}

function displaySvg(svg) {
  // display the svg on the bottom of the page for debugging
  document.body.appendChild(svg.node());
}

function markAllPoints(lat, lon) {
  for (let i = 0; i < lat.length; i++) {
    if (isNaN(lat[i]) || isNaN(lon[i])) continue;
    const marker = L.circleMarker([lat[i], lon[i]], {radius: 1}).addTo(map);
  }
}

function displayVectorField(u_key, v_key, time) {

  clearVectorField();

  // generate wind barb overlay
  let zoomLevel = map.getZoom();
  let svgOverlay = vector_field_overlay(gribBytes, u_key, v_key, BigInt(time), BigInt(zoomLevel), ArrowType.WIND_BARB);

  // maxZoomLevel is the highest zoomLevel for which an svgOverlay is generated, for all higher zoomLevels the
  // svgOverlay of maxZoomLevel is used. maxZoomLevel is the zoomLevel at which all vectors are rendered.
  const maxZoomLevel = svgOverlay.maxZoomLevel;
  zoomLevel = Math.min(zoomLevel, Number(maxZoomLevel));

  // minZoomLevel is the lowest zoomLevel for which an svgOverlay is generated, for all lower zoomLevels the
  // svgOverlay of minZoomLevel is used. minZoomLevel is the zoomLevel at the whole overlay is visible without panning.
  overlayBounds = [
    [svgOverlay.minLat, svgOverlay.minLon],
    [svgOverlay.maxLat, svgOverlay.maxLon],
  ]
  const minZoomLevel = map.getBoundsZoom(overlayBounds);

  if (zoomLevel < minZoomLevel) {
    zoomLevel = minZoomLevel;
    svgOverlay = vector_field_overlay(gribBytes, u_key, v_key, BigInt(time), BigInt(zoomLevel), ArrowType.WIND_BARB);
    // svgOverlay.maxZoomLevel, svgOverlay.minLat, ..., svgOverlay.maxLon are independent of zoomLevel
  }
  // TODO: minZoomLevel can only be determined from overlayBounds with leaflet method map.getBounds, which means that
  // the vector_field_overlay has to be redrawn when initially zoomLevel < minZoomLevel. Can we avoid this?

  // Now display the wind barbs
  const svgBlob = new Blob([svgOverlay.svgString], { type: "image/svg+xml;charset=utf-8" });
  const vecFieldUrl = URL.createObjectURL(svgBlob);

  const vecFieldBounds = [
    [svgOverlay.minLat, svgOverlay.minLon],
    [svgOverlay.maxLat, svgOverlay.maxLon],
  ]

  arrowLayers.push(L.imageOverlay(vecFieldUrl, vecFieldBounds, {opacity: 1.0}).addTo(map));

  // build a cache of layers at different zoom levels
  arrowZoomLayers = {};
  arrowZoomLayers[zoomLevel] = svgOverlay;

  for (let zl = minZoomLevel; zl <= maxZoomLevel; zl++) {
    if (zl == zoomLevel) {
      continue
    }
    const svgOverlay = vector_field_overlay(gribBytes, u_key, v_key, BigInt(time), BigInt(zl), ArrowType.WIND_BARB);
    arrowZoomLayers[zl] = svgOverlay;
  }

  // const data = get_scalar_field(gribBytes, u_key, BigInt(time));
  // markAllPoints(data.lat, data.lon);

  // console.log(arrowZoomLayers);
}

function displayHeatmap(key, time) {
  clearHeatMap();
  let imageOverlay;
  if (key == "magnitudeVectorField") {
    const selectedVectorFieldParameter = document.getElementById('vectorFieldParameterSelect').value;
    if (selectedVectorFieldParameter == "None") {
      return;
    }
    const [u_key, v_key] = selectedVectorFieldParameter.split(":")[1].split(',');
    imageOverlay = magnitude_heatmap_overlay(gribBytes, u_key, v_key, BigInt(time));

  } else {
    imageOverlay = heatmap_overlay(gribBytes, key, BigInt(time));
  }

  const canvas = document.createElement('canvas');
  canvas.width = imageOverlay.widthPx;
  canvas.height = imageOverlay.heightPx;
  const ctx = canvas.getContext('2d');
  const imageData = new ImageData(new Uint8ClampedArray(imageOverlay.image), imageOverlay.widthPx, imageOverlay.heightPx);
  ctx.putImageData(imageData, 0, 0);
  const url = canvas.toDataURL();

  const bounds = [
    [imageOverlay.minLat, imageOverlay.minLon],
    [imageOverlay.maxLat, imageOverlay.maxLon],
  ]

  heatLayer = L.imageOverlay(url, bounds, {opacity: 0.4}).addTo(map);
  overlayBounds = bounds;
  // map.fitBounds(bounds);
}

function displayParameters(time) {
  const selectedVectorFieldParameter = document.getElementById('vectorFieldParameterSelect').value;
  const selectedHeatMapParameter = document.getElementById('heatMapParameterSelect').value;

  if (selectedHeatMapParameter != "None") {
    displayHeatmap(selectedHeatMapParameter, time);
  } else {
    clearHeatMap();
  }
  if (selectedVectorFieldParameter != "None") {
    // extract u_key and v_key from `vector:<u_key>,<v_key>` format
    const [u_key, v_key] = selectedVectorFieldParameter.split(":")[1].split(',');
    if (u_key && v_key) {
      displayVectorField(u_key, v_key, time);
      // document.getElementById('output').textContent = `Displaying vector field with U key: ${u_key} and V key: ${v_key}`;
    } // TODO: else: something went wrong
    return;
  } else {
    clearVectorField();
  }
}

// Function to update the zoom level display
function updateZoomLevel() {
  const zoomLevelDiv = document.getElementById('zoom-level');
  zoomLevelDiv.textContent = `Zoom Level: ${map.getZoom()}`;

  // If arrow layers exist, update them based on zoom level
  if (arrowLayers.length > 0) {
    const maxZoom = Math.max(...Object.keys(arrowZoomLayers).map(zl => Number(zl)));
    const minZoom = Math.min(...Object.keys(arrowZoomLayers).map(zl => Number(zl)));
    const currentZoom = map.getZoom();
    if (currentZoom > maxZoom || currentZoom < minZoom) {
      return;
    }

    // Remove existing arrow layers
    arrowLayers.forEach(layer => map.removeLayer(layer));
    arrowLayers = [];

    // Add the appropriate layer for the current zoom level
    if (arrowZoomLayers[currentZoom]) {
      const svgOverlay = arrowZoomLayers[currentZoom]
      const svgBlob = new Blob([svgOverlay.svgString], { type: "image/svg+xml;charset=utf-8" });
      const url = URL.createObjectURL(svgBlob);

      const bounds = [
        [svgOverlay.minLat, svgOverlay.minLon],
        [svgOverlay.maxLat, svgOverlay.maxLon],
      ]

      arrowLayers.push(L.imageOverlay(url, bounds, {opacity: 1.0}).addTo(map));
    }
  }
}

function popupClosestGridPoint(lat, lon) {
  const selectedVectorFieldParameter = document.getElementById('vectorFieldParameterSelect').value;
  const selectedHeatMapParameter = document.getElementById('heatMapParameterSelect').value;

  let lat_out, lon_out;

  if (!gribBytes || !(selectedVectorFieldParameter || selectedHeatMapParameter)  || selectedTime === null) {
    return;
  }

  let popupContent = "";

  if (selectedVectorFieldParameter != "None") {
    // Vector field
    const [u_key, v_key] = selectedVectorFieldParameter.split(":")[1].split(',');
    // extract parameter name from heatMapParameterSelect
    const vecFieldSel = document.getElementById('vectorFieldParameterSelect');
    const parameterName = vecFieldSel.options[vecFieldSel.selectedIndex].textContent;
    if (u_key && v_key) {
      const u_data = query_grib_message_at_point(gribBytes, u_key, BigInt(selectedTime), lat, lon);
      const v_data = query_grib_message_at_point(gribBytes, v_key, BigInt(selectedTime), lat, lon);
      popupContent += `${parameterName}:<br>` +
        `&emsp;U: ${u_data.value.toFixed(2)}<br>` +
        `&emsp;V: ${v_data.value.toFixed(2)}<br>` +
        `&emsp;Speed: ${Math.sqrt(u_data.value ** 2 + v_data.value ** 2).toFixed(2)}<br>` +
        `&emsp;Direction: ${(90 - Math.atan2(v_data.value, u_data.value) * 180 / Math.PI).toFixed(2)}°<br>`;
        // TODO: should wind direction be inverted?
      lat_out = u_data.lat;
      lon_out = u_data.lon;
    }
  }
  if (selectedHeatMapParameter != "None" && selectedHeatMapParameter != "magnitudeVectorField") {
    const data = query_grib_message_at_point(gribBytes, selectedHeatMapParameter, BigInt(selectedTime), lat, lon);

    // extract parameter name from heatMapParameterSelect
    const heatMapSel = document.getElementById('heatMapParameterSelect');
    const parameterName = heatMapSel.options[heatMapSel.selectedIndex].textContent;

    popupContent += `${parameterName}: ${data.value.toFixed(2)}<br>`;
    lat_out = data.lat;
    lon_out = data.lon;
  }

  popupContent = "Closest grid point:<br>" +
    `&emsp;lat: ${lat_out.toFixed(8)}<br>` +
    `&emsp;lon: ${lon_out.toFixed(8)}<br>` + popupContent;

  // show popup with queried data
  L.popup()
    .setLatLng({ lat: lat_out, lng: lon_out })
    .setContent(popupContent)
    .openOn(map);
}

function findNextOrEqualTimestamp(timestamp, timestampArray) {
  // returns the first element form timestampArray that is bigger or equal to timestamp
  // We assume that timestampArray is sorted
  for (var i = 0; i < timestampArray.length; i++) {
    if (timestampArray[i] >= timestamp) {
      return timestampArray[i];
    }
  }
  // if all elements are smaller, return the last element
  return timestampArray.at(-1)
}

function findNextTimestamp(timestamp, timestampArray) {
  // returns the first element from timestampArray that is bigger than timestamp
  // We assume that timestampArray is sorted
  for (var i = 0; i < timestampArray.length; i++) {
    if (timestampArray[i] > timestamp) {
      return timestampArray[i];
    }
  }
  // if all elements are smaller, return the last element
  return timestampArray.at(-1)
}

function findPreviousTimestamp(timestamp, timestampArray) {
  // returns the biggest element from timestampArray that is smaller than timestamp
  // We assume that timestampArray is sorted
  for (var i = timestampArray.length - 1; i >= 0; i--) {
    if (timestampArray[i] < timestamp) {
      return timestampArray[i];
    }
  }
  // if all elements are bigger, return the first element
  return timestampArray.at(0)
}

init().then(() => {
  // epoch time in seconds instead of miliseconds
  selectedTime = Math.floor(Date.now() / 1000);
  map = L.map('map').setView([0, 0], 2);
  L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
    attribution: '&copy <a href="https://openstreetmap.org/copyright">OpenStreetMap</a> contributors'
  }).addTo(map);

  document.getElementById('fileInput').addEventListener('change', async (e) => {
    const file = e.target.files[0];
    if (!file) return;
    await showParameterSelect(file);
    map.fitBounds(overlayBounds);
  });

  document.getElementById('heatMapParameterSelect').addEventListener('change', (e) => {
    const selectedHMParameter = e.target.value;
    const selectedVFParameter = document.getElementById('vectorFieldParameterSelect').value;
    if ((selectedVFParameter == "None") && selectedHMParameter != "magnitudeVectorField") {
      // if no vector field is selected: timeselect of heatmap
      showTimeSelect(selectedHMParameter);
    } else if (selectedVFParameter != "None"){
      // if vectorfield is selected: show heatmap at current time
      displayParameters(selectedTime);
      // TODO: what if selectedHMParameter does not have a message with selectedTime?
    }
  });

  document.getElementById('vectorFieldParameterSelect').addEventListener('change', (e) => {
    const selectedVFParameter = e.target.value;
    const selectedHMParameter = document.getElementById('heatMapParameterSelect').value;
    if (selectedVFParameter != "None") {
      const option = document.getElementById("magnitudeVectorFieldOption").disabled = false;
      showTimeSelect(selectedVFParameter);
    } else {
      if  (selectedHMParameter == "magnitudeVectorField") {
        document.getElementById('heatMapParameterSelect').value = "None";
      }
      const option = document.getElementById("magnitudeVectorFieldOption").disabled = true;
      showTimeSelect(selectedHMParameter);
    }
  });

  document.getElementById('timestampSelect').addEventListener('change', (e) => {
    selectedTime = Number(e.target.value);
    displayParameters(selectedTime);
  });

  document.getElementById('nowTimestampButton').addEventListener("click", () => {
    const select = document.getElementById('timestampSelect');
    const availableTimes = Array.from(select.options).map(((o) => Number(o.value)));
    selectedTime = findNextOrEqualTimestamp(Math.floor(Date.now() / 1000), availableTimes);
    select.value = selectedTime;
    displayParameters(selectedTime);
  });

  document.getElementById('nextTimestampButton').addEventListener("click", () => {
    const select = document.getElementById('timestampSelect');
    const availableTimes = Array.from(select.options).map(((o) => Number(o.value)));
    selectedTime = findNextTimestamp(selectedTime, availableTimes);
    select.value = selectedTime;
    displayParameters(selectedTime);
  });

  document.getElementById('prevTimestampButton').addEventListener("click", () => {
    const select = document.getElementById('timestampSelect');
    const availableTimes = Array.from(select.options).map(((o) => Number(o.value)));
    selectedTime = findPreviousTimestamp(selectedTime, availableTimes);
    select.value = selectedTime;
    displayParameters(selectedTime);
  });

  map.on('click', function(e) {
    const lat = e.latlng.lat;
    const lon = e.latlng.lng;
    popupClosestGridPoint(lat, lon)
  });

  // Initial update
  updateZoomLevel();

  // Update on zoom end
  map.on('zoomend', updateZoomLevel);
});
