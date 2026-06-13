import L from "leaflet";

// tick positions
const TICK_POSITIONS = [0, 0.2, 0.4, 0.6, 0.8, 1];

class ColorbarControl extends L.Control {
  colorbarImageUrl: string;
  minValue: number;
  maxValue: number;

  // options: L.ControlOptions

  constructor(
    colorbarImageUrl: string,
    minValue: number,
    maxValue: number,
    options?: L.ControlOptions,
  ) {
    super(options);
    this.colorbarImageUrl = colorbarImageUrl;
    this.minValue = minValue;
    this.maxValue = maxValue;
    // this.options = options;
  }

  onAdd(_map: L.Map): HTMLElement {
    let div = L.DomUtil.create("div", "colorbar-control") as HTMLDivElement;


    // Container for image + ticks
    const container = L.DomUtil.create("div", "colorbar-container", div);

    // Colorbar image
    const img = L.DomUtil.create(
      "img",
      "colorbar-img",
      container,
    ) as HTMLImageElement;
    img.src = this.colorbarImageUrl;

    // Tick marks container
    const ticks = L.DomUtil.create("div", "colorbar-ticks", container);

    TICK_POSITIONS.forEach((v, i) => {
      const tick = L.DomUtil.create("div", "tick", ticks);

      // Compute position (%) from index
      const pos = (i / (TICK_POSITIONS.length - 1)) * 100;
      tick.style.left = `${pos}%`;

      tick.innerHTML = `<span class="tick-label">${(v * (this.maxValue - this.minValue) + this.minValue).toFixed(2)}</span>`;
    });

    return div;
  }

  onRemove(_map: L.Map): void {}
}

function createColorBar(
  colorbarImageUrl: string,
  minValue: number,
  maxValue: number,
  options?: L.ControlOptions,
) {
  return new ColorbarControl(colorbarImageUrl, minValue, maxValue, options);
}

export { createColorBar, ColorbarControl };
