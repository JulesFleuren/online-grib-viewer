# Online-Grib-Viewer

**Online-Grib-Viewer** is a tool for visualizing Grib2-files, right in your browser. It is hosted at [gribviewer.com](https://gribviewer.com).

The website runs completely client side. All data processing is done inside your browser, and no data is sent to a server. No software needs to be installed, except for a browser.

## Capabilities

**Online-Grib-Viewer** has the following capabilities:

- Display data on a slippy map, allowing zooming and panning around
- Selecting a Grib message based on parameter, timestamp and fixed surfaces
- Visualizing Grib messages as a heatmap
- Visualizing Grib messages as a vectorfield (with either barbs or arrows)
- Displaying basic data about a Grib message
- Support for Latitude-Longitude grids and Gaussian grids

The following capabilities are not (yet) supported:

- Reading Grib1 files
- Reading Grib ensemble files
- Reading Grib messages on other grids than lat-lon or Gaussian
- ...

## Contributing

**Online-Grib-Viewer** is still in early development. If you find any bugs, or have a feature request, don't hesitate to make an issue. Be sure to follow the issue templates. You are also welcome to fork the repository and create a pull request.

## Installation

First, make sure npm and cargo are installed. Clone the repo and open a terminal in the root folder of the project. Then install the npm dependencies:

```
npm install
```

Install wasm-pack:

```
cargo install wasm-pack
```

## Development

After installation, we can compile the rust-wasm code:

```
npm run wasm
```

And run the dev server:

```
npm run dev
```

Each time the rust code is changed, `npm run wasm` has to be run. The typescript code will automatically be updated when the file is saved.

To load the Protomaps basemap, a Protomaps api-key is needed. You can get one by signing up on the [Protomaps website](https://protomaps.com/api). Copy the `.env.example` file at the root of this repo, rename it to `.env`, and replace the dummy key with your own key. The basemap should now work. (I am planning to also support an offline basemap in the form of a `.pmtiles` file, but this is not yet implemented.)

### Testing

Before making a pull request, check if all the (rust) tests pass:

```
npm run test:rust
```

and if the typescript typecheck passes:

```
npm run typecheck
```
