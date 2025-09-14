import { getWindBarb } from './svg-wind-barbs/js/GetWindBarb.js';
import { generate_wind_barbs_svg_overlay, generate_projected_wind_barbs_svg_overlay } from './pkg/online_grib_viewer.js';

function generateHeatMapCanvas(nx, ny, values) {
  const canvas = document.createElement('canvas');
  canvas.width = nx;
  canvas.height = ny;
  const ctx = canvas.getContext('2d');
  const imageData = ctx.createImageData(canvas.width, canvas.height);
  const data = imageData.data;

  // Find min and max values for normalization
  const validValues = Array.from(values).filter(v => !isNaN(v));
  const minValue = Math.min(...validValues);
  const maxValue = Math.max(...validValues);

  // Invert the loop: draw from bottom row to top row
  for (let y = 0; y < canvas.height; y++) {
    for (let x = 0; x < canvas.width; x++) {
      const srcY = canvas.height - 1 - y; // invert y
      const i = srcY * canvas.width + x;
      const value = values[i];
      const normalized = (value - minValue) / (maxValue - minValue);
      const color = valueToColor(normalized);

      const idx = (y * canvas.width + x) * 4;
      data[idx] = color.r;
      data[idx + 1] = color.g;
      data[idx + 2] = color.b;
      data[idx + 3] = isNaN(value) ? 0 : 255;
    }
  }

  ctx.putImageData(imageData, 0, 0);
  return canvas;
}

function generateHeatMapSvg(nx, ny, lat, lon, values) {
  // Create a new SVG element
  const svg = d3.create("svg")

  const cellSize = 1;

  // Find min and max values for normalization
  const validValues = Array.from(values).filter(v => !isNaN(v));
  const minValue = Math.min(...validValues);
  const maxValue = Math.max(...validValues);

  const colorScale = d3.scaleSequential()
    .domain([minValue, maxValue])
    .interpolator(d3.interpolateViridis);

  const data = [];
  for (let i = 0; i < ny; i++) {
    for (let j = 0; j < nx; j++) {
      const idx = i * nx + j;
      if (isNaN(values[idx])) continue;
      data.push({x: j, y: ny - i - 1, value: values[idx]});
    }
  }

  svg.selectAll("rect")
    .data(data)
    .enter()
    .append("rect")
    .attr("class", "cell")
    .attr("x", d => d.x * cellSize)
    .attr("y", d => d.y * cellSize)
    .attr("width", cellSize)
    .attr("height", cellSize)
    .attr("fill", d => colorScale(d.value));

  svg.attr("viewBox", `0 0 ${nx * cellSize} ${ny * cellSize}`);

  // all cells are made as squares, so we need to disable aspect ratio preservation,
  // because the spacing of the grid is not necessarily square
  svg.node().setAttribute("preserveAspectRatio", "none");
  return svg;
}

function generateWindBarbSvg(lat, lon, nlat, nlon, u, v, zoomLevel) {
  
  const svg_string = generate_projected_wind_barbs_svg_overlay(lat, lon, BigInt(nlat), BigInt(nlon), u, v, BigInt(zoomLevel))

  console.log(svg_string)  
  const parser = new DOMParser();
  const doc = parser.parseFromString(svg_string, "image/svg+xml");
  const svg = doc.documentElement;
  
  // TODO: the barbs are not scaled correctly for non-square grids
  // TODO: the barbs are not centered in all cells due to projection issues
  return svg;
}

function valueToColor(value) {
  const color = {r: value*255, g: (1-value)*255, b: 0};
  return color;
}

export { generateHeatMapCanvas, generateHeatMapSvg, generateWindBarbSvg };