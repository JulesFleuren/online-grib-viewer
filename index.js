import init, { get_available_parameters,
  get_available_timestamps,
  query_grib_message_at_point,
  vector_field_overlay,
  heatmap_overlay,
  magnitude_heatmap_overlay,
  get_scalar_field} from './pkg/online_grib_viewer.js';

// certain parameter pairs are known to be vector fields
const PARAMETER_PAIRS = {
  'current': { u: "grib2_10_1_2", v: "grib2_10_1_3" },
  'wind': { u: "grib2_0_2_2", v: "grib2_0_2_3" },
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
let selectedParameter = null;
let selectedTime = null;
let overlayBounds = null;

function clearMap() {
  if (heatLayer) {
    map.removeLayer(heatLayer);
    heatLayer = null;
  }
  arrowLayers.forEach(layer => map.removeLayer(layer));
  arrowLayers = [];
}

async function showParameterSelect(file) {
  const arrayBuffer = await file.arrayBuffer();
  gribBytes = new Uint8Array(arrayBuffer);

  parameters = get_available_parameters(gribBytes)

  console.log('Available parameters:', parameters);

  // add all available parameters as option to select object
  const select = document.getElementById('parameterSelect');
  select.innerHTML = '';
  parameters.forEach(p => {
    const option = document.createElement('option');
    option.value = p.key;
    option.textContent = p.name;
    select.appendChild(option);
  });

  // Check if any parameter pairs are available for vector field display and add an option if so
  Object.keys(PARAMETER_PAIRS).forEach(pairName => {
    const pair = PARAMETER_PAIRS[pairName];
    const hasU = parameters.some(p => p.key === pair.u);
    const hasV = parameters.some(p => p.key === pair.v);
    if (hasU && hasV) {
      const option = document.createElement('option');
      option.value = `vector:${pair.u},${pair.v}`;
      option.textContent = `${pairName} (vector field)`;
      select.appendChild(option);
    }
  });

  document.getElementById('parameterField').style.display = '';

  // automatically select the first parameter and show the timesteps
  if (parameters.length > 0) {
    showTimeSelect(parameters[0].key);
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
    displayParameter(selectedTime);
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

  clearMap();

  const imageOverlay = magnitude_heatmap_overlay(gribBytes, u_key, v_key, BigInt(time));
  const canvas = document.createElement('canvas');
  canvas.width = imageOverlay.width;
  canvas.height = imageOverlay.height;
  const ctx = canvas.getContext('2d');
  const imageData = new ImageData(new Uint8ClampedArray(imageOverlay.image), imageOverlay.width, imageOverlay.height);
  ctx.putImageData(imageData, 0, 0);
  const heatmapUrl = canvas.toDataURL();

  const heatmapBounds = [
    [imageOverlay.minLat, imageOverlay.minLon],
    [imageOverlay.maxLat, imageOverlay.maxLon],
  ]

  heatLayer = L.imageOverlay(heatmapUrl, heatmapBounds, {opacity: 0.4}).addTo(map);

  overlayBounds = heatmapBounds;
  // map.fitBounds(overlayBounds);

  // generate wind barb overlay
  const minZoomLevel = map.getBoundsZoom(overlayBounds);
  let zoomLevel = map.getZoom();
  zoomLevel = Math.max(zoomLevel, minZoomLevel);
  const svgOverlay = vector_field_overlay(gribBytes, u_key, v_key, BigInt(time), BigInt(zoomLevel), ArrowType.PIVOT_CENTER);
  const maxZoomLevel = svgOverlay.maxZoomLevel;
  zoomLevel = Math.min(zoomLevel, Number(maxZoomLevel));

  // Now display the wind barbs
  const svgBlob = new Blob([svgOverlay.svgString], { type: "image/svg+xml;charset=utf-8" });
  const windBarbUrl = URL.createObjectURL(svgBlob);

  const windBarbBounds = [
    [svgOverlay.minLat, svgOverlay.minLon],
    [svgOverlay.maxLat, svgOverlay.maxLon],
  ]

  arrowLayers.push(L.imageOverlay(windBarbUrl, windBarbBounds, {opacity: 1.0}).addTo(map));

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
  clearMap();

  const imageOverlay = heatmap_overlay(gribBytes, key, BigInt(time));
  const canvas = document.createElement('canvas');
  canvas.width = imageOverlay.width;
  canvas.height = imageOverlay.height;
  const ctx = canvas.getContext('2d');
  const imageData = new ImageData(new Uint8ClampedArray(imageOverlay.image), imageOverlay.width, imageOverlay.height);
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

function displayParameter(time) {
  const paramSelect = document.getElementById('parameterSelect');
  selectedParameter = paramSelect.value;
  if (selectedParameter.startsWith('vector')) {
    // Vector field
    const [u_key, v_key] = selectedParameter.split(":")[1].split(',');
    if (u_key && v_key) {
      displayVectorField(u_key, v_key, time);

      document.getElementById('output').textContent = `Displaying vector field with U key: ${u_key} and V key: ${v_key}`;
    }
    return;
  } else {
      // Scalar field
      const key = selectedParameter;
      if (key) {
        displayHeatmap(key, time);

        document.getElementById('output').textContent = `Displaying parameter at key ${key}`;
      }
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

  document.getElementById('parameterSelect').addEventListener('change', (e) => {
    selectedParameter = e.target.value;
    // check if the selected value is a vector field
    showTimeSelect(selectedParameter);
  });

  document.getElementById('timestampSelect').addEventListener('change', (e) => {
    selectedTime = Number(e.target.value);
    displayParameter(selectedTime);
  });

  document.getElementById('nowTimestampButton').addEventListener("click", () => {
    const select = document.getElementById('timestampSelect');
    const availableTimes = Array.from(select.options).map(((o) => Number(o.value)));
    selectedTime = findNextOrEqualTimestamp(Math.floor(Date.now() / 1000), availableTimes);
    select.value = selectedTime;
    displayParameter(selectedTime);
  });

  document.getElementById('nextTimestampButton').addEventListener("click", () => {
    const select = document.getElementById('timestampSelect');
    const availableTimes = Array.from(select.options).map(((o) => Number(o.value)));
    selectedTime = findNextTimestamp(selectedTime, availableTimes);
    select.value = selectedTime;
    displayParameter(selectedTime);
  });

  document.getElementById('prevTimestampButton').addEventListener("click", () => {
    const select = document.getElementById('timestampSelect');
    const availableTimes = Array.from(select.options).map(((o) => Number(o.value)));
    selectedTime = findPreviousTimestamp(selectedTime, availableTimes);
    select.value = selectedTime;
    displayParameter(selectedTime);
  });

  map.on('click', function(e) {
    const lat = e.latlng.lat;
    const lng = e.latlng.lng;

    if (!gribBytes || !selectedParameter || selectedTime === null) {
      return;
    }

    if (selectedParameter.startsWith('vector')) {
      // Vector field
      const [u_key, v_key] = selectedParameter.split(":")[1].split(',');
      if (u_key && v_key) {
        const u_data = query_grib_message_at_point(gribBytes, u_key, BigInt(selectedTime), lat, lng);
        const v_data = query_grib_message_at_point(gribBytes, v_key, BigInt(selectedTime), lat, lng);

        // show popup with queried data
        L.popup()
          .setLatLng({lat: u_data.lat, lng: u_data.lon})
          .setContent("Closest grid point:<br>" +
                      `Lat: ${u_data.lat.toFixed(5)}<br>Lng: ${u_data.lon.toFixed(5)}<br>` +
                      `U: ${u_data.value.toFixed(2)}<br>` +
                      `V: ${v_data.value.toFixed(2)}<br>` +
                      `Speed: ${Math.sqrt(u_data.value**2 + v_data.value**2).toFixed(2)}<br>` +
                      `Direction: ${(180 + 90 - Math.atan2(v_data.value, u_data.value) * 180 / Math.PI).toFixed(2)}°<br>`
                    )
          .openOn(map);

        // TODO: wind direction is inverted
        return;
      }
    } else {
      // Scalar field
      const key = selectedParameter;
      const data = query_grib_message_at_point(gribBytes, key, BigInt(selectedTime), lat, lng);

      // show popup with queried data
      L.popup()
        .setLatLng({lat: data.lat, lng: data.lon})
        .setContent("Closest grid point:<br>" +
                    `Lat: ${data.lat.toFixed(5)}<br>Lng: ${data.lon.toFixed(5)}<br>` +
                    `${key}: ${data.value.toFixed(2)}<br>`
                  )
        .openOn(map);
    }
  });

  // Initial update
  updateZoomLevel();

  // Update on zoom end
  map.on('zoomend', updateZoomLevel);
});
