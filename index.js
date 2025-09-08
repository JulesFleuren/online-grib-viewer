import init, { get_available_parameters, get_vector_field, get_available_timestamps, get_grib_index, get_scalar_field, get_grid_shape } from './pkg/grib_reader_wasm.js';
import { generateHeatMapCanvas, generateHeatMapSvg, generateWindBarbSvg } from './map-overlays.js';

// certain parameter pairs are known to be vector fields
const PARAMETER_PAIRS = {
  'current': { u: { discipline: 10, category: 1, parameter: 2 }, v: { discipline: 10, category: 1, parameter: 3 } },
};

let map;
let heatLayer;
let arrowLayers = [];
let gribBytes = null;
let parameters = [];
let selectedParameter = null;
let selectedTime = null;

function clearMap() {
  if (heatLayer) {
    map.removeLayer(heatLayer);
    heatLayer = null;
  }
  arrowLayers.forEach(layer => map.removeLayer(layer));
  arrowLayers = [];
}

function showParameterSelect(params) {
  const select = document.getElementById('parameterSelect');
  select.innerHTML = '';
  params.forEach(p => {
    const name = p.name;
    const discipline = p.discipline;
    const category = p.category;
    const parameter = p.parameter;
    const option = document.createElement('option');
    option.value = `${discipline},${category},${parameter}`;
    option.textContent = name;
    select.appendChild(option);
  });

  // Check if any parameter pairs are available for vector field display and add an option if so
  Object.keys(PARAMETER_PAIRS).forEach(pairName => {
    const pair = PARAMETER_PAIRS[pairName];
    const hasU = params.some(p => p.discipline === pair.u.discipline && p.category === pair.u.category && p.parameter === pair.u.parameter);
    const hasV = params.some(p => p.discipline === pair.v.discipline && p.category === pair.v.category && p.parameter === pair.v.parameter);
    if (hasU && hasV) {
      const option = document.createElement('option');
      option.value = `vector,${pair.u.discipline},${pair.u.category},${pair.u.parameter},${pair.v.discipline},${pair.v.category},${pair.v.parameter}`;
      option.textContent = `${pairName} (vector field)`;
      select.appendChild(option);
    }
  });

  document.getElementById('parameterField').style.display = '';
}

function showTimeSelect(times) {
  const select = document.getElementById('timestampSelect');
  select.innerHTML = '';
  times.forEach(t => {
    const option = document.createElement('option');
    option.value = t;
    option.textContent = new Date(Number(t) * 1000).toISOString();
    select.appendChild(option);
  });
  document.getElementById('timestampField').style.display = '';
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

function displayVectorField(nx, ny, lat, lon, u, v) {
  clearMap();

  const bounds = [[Math.min(...lat), Math.min(...lon)], [Math.max(...lat), Math.max(...lon)]];

  // the svg extends half a cellsize beyond the bounds, so we need to adjust the bounds accordingly
  const cellSizeLat = (Math.max(...lat) - Math.min(...lat)) / ny;
  const cellSizeLon = (Math.max(...lon) - Math.min(...lon)) / nx;

  const adjustedBounds = [
    [bounds[0][0] - cellSizeLat / 2, bounds[0][1] - cellSizeLon / 2],
    [bounds[1][0] + cellSizeLat / 2, bounds[1][1] + cellSizeLon / 2]
  ];

  const uArray = Array.from(u);

  // Also display a heatmap of the norm of the vector field
  const normValues = u.map((val, idx) => {
    if (isNaN(val) || isNaN(v[idx])) return NaN;
    return Math.sqrt(val*val + v[idx]*v[idx]);
  });

  const heatmapSvg = generateHeatMapSvg(nx, ny, lat, lon, normValues);
  heatLayer = L.svgOverlay(heatmapSvg.node(), adjustedBounds, {opacity: 0.6}).addTo(map);

  // Now display the wind barbs 
  const svg = generateWindBarbSvg(nx, ny, lat, lon, u, v, 0.1);
  
  arrowLayers.push(L.svgOverlay(svg.node(), adjustedBounds).addTo(map));
}

function displayHeatmap(nx, ny, lat, lon, values) {
  clearMap();
  
  const bounds = [[Math.min(...lat), Math.min(...lon)], [Math.max(...lat), Math.max(...lon)]];
  // the svg extends half a cellsize beyond the bounds, so we need to adjust the bounds accordingly
  const cellSizeLat = (Math.max(...lat) - Math.min(...lat)) / ny;
  const cellSizeLon = (Math.max(...lon) - Math.min(...lon)) / nx;

  const adjustedBounds = [
    [bounds[0][0] - cellSizeLat / 2, bounds[0][1] - cellSizeLon / 2],
    [bounds[1][0] + cellSizeLat / 2, bounds[1][1] + cellSizeLon / 2]
  ];
  
  const svg = generateHeatMapSvg(nx, ny, lat, lon, values);

  heatLayer = L.svgOverlay(svg.node(), adjustedBounds, {opacity: 0.6}).addTo(map);

  map.fitBounds(bounds);
}

function displayParameter(time) {
  const paramSelect = document.getElementById('parameterSelect');
  const selectedValue = paramSelect.value;
  if (selectedValue.startsWith('vector')) {
    const [_, disciplineU, categoryU, parameterU, disciplineV, categoryV, parameterV] = selectedValue.split(',').map(Number);
    const u_index = get_grib_index(gribBytes, disciplineU, categoryU, parameterU, BigInt(time));
    const v_index = get_grib_index(gribBytes, disciplineV, categoryV, parameterV, BigInt(time));
    if (u_index && v_index) {
      const data = get_vector_field(gribBytes, u_index.index, v_index.index, u_index.subindex, v_index.subindex);
      const shape = get_grid_shape(gribBytes, u_index.index, u_index.subindex);
      displayVectorField(shape.nx, shape.ny, data.lat, data.lon, data.u, data.v);

      document.getElementById('output').textContent = `Displaying vector field with U at index ${u_index.index}, subindex ${u_index.subindex} and V at index ${v_index.index}, subindex ${v_index.subindex}`;
    }
    return;
  } else {
      // Scalar field
      const [discipline, category, parameter] = selectedValue.split(',');
      const index = get_grib_index(gribBytes, discipline, category, parameter, BigInt(time));

      if (index) {
        // If we found the index, we can display the heatmap
        const data = get_scalar_field(gribBytes, index.index, index.subindex);
        const shape = get_grid_shape(gribBytes, index.index, index.subindex);
        displayHeatmap(shape.nx, shape.ny, data.lat, data.lon, data.values);

        document.getElementById('output').textContent = `Displaying parameter at index ${index.index}, subindex ${index.subindex}`;  
      }
  }
}

init().then(() => {
  map = L.map('map').setView([0, 0], 2);
  L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png').addTo(map);

  document.getElementById('fileInput').addEventListener('change', async (e) => {
    const file = e.target.files[0];
    if (!file) return;
    const arrayBuffer = await file.arrayBuffer();
    gribBytes = new Uint8Array(arrayBuffer);

    parameters = get_available_parameters(gribBytes).map(p => ({
      name: p.name,
      discipline: p.discipline,
      category: p.category,
      parameter: p.parameter
    }));

    console.log('Available parameters:', parameters);

    showParameterSelect(parameters);
    document.getElementById('output').textContent = `Loaded ${parameters.length} parameters.`;
  });

  document.getElementById('parameterSelect').addEventListener('change', (e) => {
    selectedParameter = e.target.value;
    // check if the selected value is a vector field
    if (e.target.value.startsWith('vector')) {
      const [disciplineU, categoryU, parameterU, disciplineV, categoryV, parameterV] = e.target.value.split(',').slice(1).map(Number);
      var timesU = get_available_timestamps(gribBytes, disciplineU, categoryU, parameterU);
      var timesV = get_available_timestamps(gribBytes, disciplineV, categoryV, parameterV);
      // find intersection of timesU and timesV
      var times = timesU.filter(t => timesV.includes(t));
      showTimeSelect(times);
    } else {
      // otherwise, it's a scalar field
      const [discipline, category, parameter] = e.target.value.split(',');
      var times = get_available_timestamps(gribBytes, Number(discipline), Number(category), parameter);
      showTimeSelect(times);
    }

  });

  document.getElementById('timestampSelect').addEventListener('change', (e) => {
    selectedTime = Number(e.target.value);
    displayParameter(selectedTime);
  });
});
