# Online-Grib-Viewer

**Online-Grib-Viewer** is a tool for visualizing Grib2-files, right in your browser. It is hosted at [gribviewer.com](https://gribviewer.com).

The website runs completely client side. All data processing is done inside your browser, and no data is sent to a server. No software needs to be installed, except for a browser. It can also be run offline, without an internet connection (see [Offline Grib Viewer](#offline-grib-viewer)).

## Capabilities

**Online-Grib-Viewer** has the following capabilities:

- Display data on a slippy map, allowing zooming and panning around
- Selecting a Grib message based on parameter, timestamp and fixed surfaces
- Visualizing Grib messages as a heatmap
- Visualizing Grib messages as a vector field (with either barbs or arrows)
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

```bash
npm install
```

Install wasm-pack:

```bash
cargo install wasm-pack
```

## Development

After installation, we can compile the rust-wasm code:

```bash
npm run wasm
```

And run the dev server:

```bash
npm run dev
```

Each time the rust code is changed, `npm run wasm` has to be run. The typescript code will automatically be updated when the file is saved.

To load the Protomaps basemap, a Protomaps api-key is needed. You can get one by signing up on the [Protomaps website](https://protomaps.com/api). Copy the `.env.example` file at the root of this repo, rename it to `.env`, and replace the dummy key with your own key. The basemap should now work. Using an offline basemap (see [Offline Grib Viewer](#offline-grib-viewer)) is also possible. For development, place it in `/public` and modify `/public/settings`.

### Testing

Before making a pull request, check if all the (rust) tests pass:

```bash
npm run test:rust
```

and if the typescript type check passes:

```bash
npm run typecheck
```

## Tech stack

The web app is written in Typescript, and Rust, compiled to WebAssembly. Reading the Grib file, and generating the heatmap and vector field overlays, is done in the Rust code. For this, it makes extensive use of the [grib-rs](https://docs.rs/crate/grib/latest) crate. Most other functionality is implemented in TypeScript. The slippy map makes use of [Leaflet](https://leafletjs.com/), with basemaps from [Protomaps](https://protomaps.com/), based on data from [OpenStreetMap](https://www.openstreetmap.org/about). [Bulma CSS](https://bulma.io/) is used for the styling. [Vite](https://vite.dev/) is used to build and bundle the app.

## Offline Grib Viewer

It is possible to use Online Grib Viewer without any internet connection, after downloading some files. This section describes how to set this up.

1. Download the latest release from GitHub. This can be found under the "Releases" header on the right of the main page of the GitHub repository, or via [this link](https://github.com/JulesFleuren/online-grib-viewer/releases). Only the `dist-<version>.zip` file is required. Download the zip file and extract it to a folder of your choosing. We will assume it is extracted to `/path-to/dist-<version>`. The directory `/path-to/dist-<version>` should contain the file `index.html` and two subdirectories: `assets` and `settings`.
2. By default, basemaps from the protomaps API will be used. The site can be used like this, but it still requires an internet connection to load the basemap. For offline use, a basemap has to be downloaded. This can be done in the form of a `.pmtiles` file, for example from [https://maps.protomaps.com/builds/](https://maps.protomaps.com/builds/). Basemaps containing the whole world with all zoom levels can be considerable (>120 GB), so my advice is to extract only the areas and zoom levels that are of interest. See [Extracting area of basemap](#extracting-area-of-basemap) for an explanation. Place the basemap in `/path-to/dist-<version>`.
3. Add the `url` entry to the file `/path-to/dist-<version>/settings/basemapSettings.json`. If the file that was downloaded in step 2, is called `map.pmtiles`, then the entry should look as follows:

   ```json
   {
     "url": "map.pmtiles"
   }
   ```

   Other settings, such as the style and language of the map, can also be changed with this file. See [Basemap Settings](#basemap-settings) for more details.

4. (Optional) Change any of the other settings in the `/path-to/dist-<version>/settings` folder. See [Settings](#settings) for all the options.
5. Serve the files with an http-server that supports HTTP range requests, with `/path-to/dist-<version>` as root. See [#HTTP Server](#http-server) for more details.

### Extracting area of basemap

Extracting part of a basemap can be done with the `pmtiles` tool, as described in [https://docs.protomaps.com/guide/getting-started](https://docs.protomaps.com/guide/getting-started). The `maxDataZoom` and `bounds` attributes in the `basemapSettings.json` have to be set to match the settings of the extraction.

#### Example:

Say we want to extract an area containing The Netherlands, and we would like to be able to zoom up to level 11 (this is also the default that [gribviewer.com](gribviewer.com) uses). We check [https://maps.protomaps.com/builds/](https://maps.protomaps.com/builds/), and see that `20260126.pmtiles` is the latest build. We use the following command to download the extracted data (make sure the pmtiles tool is installed: [https://docs.protomaps.com/guide/getting-started](https://docs.protomaps.com/guide/getting-started)):

```bash
pmtiles extract https://build.protomaps.com/20260126.pmtiles "/path-to/dist-<version>/netherlands_z11.pmtiles" --bbox=3.175049,50.625073,7.459717,53.644638 --maxzoom=11
```

You can, of course, change the name `netherlands_z11.pmtiles`, to anything you like. Now we modify `/path-to/dist-<version>/settings/basemapSettings.json` so that it looks as follows:

```json
{
  "url": "netherlands_z11.pmtiles",
  "maxDataZoom": 11,
  "bounds": [
    { "lng": 3.175049, "lat": 50.625073 },
    { "lng": 7.459717, "lat": 53.644638 }
  ]
}
```

### HTTP Server

For offline use with a local `mbtiles` basemap, an http server that supports HTTP range requests is required. Here we list a few options:

- [https://github.com/http-party/http-server](https://github.com/http-party/http-server)
- [https://github.com/danvk/RangeHTTPServer](https://github.com/danvk/RangeHTTPServer)
- [https://static-web-server.net/](https://static-web-server.net/)

If you haven't already installed python or node.js, the last option is probably the easiest to install, since a precompiled binary is available.

#### Static Web Server

Download the binary for your platform from [https://static-web-server.net/download-and-install/](https://static-web-server.net/download-and-install/). Unzip the folder and copy the file `static-web-server` (or `static-web-server.exe` if you're on Windows) to `/path-to/dist-<version>`. Now open a terminal in `/path-to/dist-<version>` and run `./static-web-server --port 8000 --root .` (or `static-web-server.exe --port 8000 --root .` on windows). Open your browser and visit `localhost:8000`. For more information visit the [Static Web Server documentation](https://static-web-server.net/).

## Settings

Some settings can be changed trough the three config files in the `settings` folder. The files are in `json` format.

### Overlay Settings

These settings change the appearance of the vector field and heatmap overlays. The settings can be provided as a key-value pair, where the key is a string representing the parameter, and the value is an object specifying the settings for that parameter.

<!--TODO: explain grib key strings-->

#### Properties

| Property            | Type                                            | Default      | Description                                                                                                                                            |
| ------------------- | ----------------------------------------------- | ------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `colorMin` *        | `number` \| `"FileBased"` \| `"MessageBased"`   | `FileBased`  | Minimum value for color scale. If `"FileBased"`, use the file's min. If `"MessageBased"`, use the message's min.                                       |
| `colorMax` *        | `number` \| `"FileBased"` \| `"MessageBased"`   | `FileBased`  | Maximum value for color scale. If `"FileBased"`, use the file's max. If `"MessageBased"`, use the message's max.                                       |
| `removeOutOfBounds` * | `boolean`                                       | `false`      | If `true`, values that fall outside of the interval [`colorMin`, `colorMax`] are transparent; if `false`, values are clamped to `colorMin`/`colorMax`. |
| `pixelsPerPoint` *    | `number`                                        | `3`          | Controls the resolution of the heatmap overlay.                                                                                                        |
| `arrowType` **        | `"PivotTip"` \| `"PivotCenter"` \| `"WindBarb"` | `"PivotTip"` | Arrow style for vector data.                                                                                                                           |
| `scaleArrow` **       | `boolean`                                       | `false`      | If `true`, scale arrow size by magnitude.                                                                                                              |
| `scaleMax` **         | `number` \| `"FileBased"` \| `"MessageBased"`   | `FileBased`  | Maximum value for arrow scaling. If `"FileBased"`, use the file's max. If `"MessageBased"`, use the message's max.                                     |

\* Applies to heatmaps only.

\*\* Applies to vector fields only.

**Note:**

- Omitted properties will use default values.
- `"FileBased"`: the maximum and/or minimum values are calculated from the file when a message for that parameter is first selected.

#### Example

```json
{
  "vector:grib2_0_2_2,grib2_0_2_3": {
    "arrowType": "WindBarb",
    "scaleArrow": false,
    "colorMin": 0,
    "colorMax": 35.0,
    "removeOutOfBounds": false
  },
  ...
}
```

Here we see an entry defining settings for the vector pair `vector:grib2_0_2_2,grib2_0_2_3` (which is wind data).

### Basemap Settings

These settings control the appearance and behavior of the basemap layer. The basemap is loaded from either the Protomaps API or a local `.pmtiles` file. Settings can be customized by creating or modifying the `settings/basemapSettings.json` file.

#### Properties

| Property      | Type                    | Default      | Description                                                                                                                                         |
| ------------- | ----------------------- | ------------ | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| `url`         | `string`                | (required)   | URL to the basemap tiles. Can be a Protomaps API endpoint or a path to a local `.pmtiles` file.                                                   |
| `flavor`      | `string`                | `"light"`     | Map style. Common options include `"light"`, `"dark"`, `"grayscale"`, etc.                                                                      |
| `lang`        | `string`                | `"en"`        | Language for map labels. Use ISO 639-1 language codes (e.g., `"en"`, `"nl"`, `"de"`).                                                          |
| `maxDataZoom` | `number`                | `11`         | Maximum zoom level available in the tile data. Must match the zoom level of your `.pmtiles` file when using local basemaps.                      |
| `maxZoom`     | `number`                | `22`         | Maximum zoom level the user can zoom to. Can exceed `maxDataZoom`; tiles will be upsampled if needed.                                            |
| `bounds`      | `LatLngBoundsExpression` | Not set      | Geographic bounds of the basemap. Restricts the map area. Useful when using extracted `.pmtiles` files with limited coverage.                    |

**Note:**

- Omitted properties will use default values.
- For offline use with a local `.pmtiles` file, ensure `maxDataZoom` matches the zoom level of your extracted file.
- The `bounds` property is useful when you've extracted a regional `.pmtiles` file to avoid panning to areas with missing data.

#### Examples

**API-based basemap (online):**

```json
{
  "url": "https://api.protomaps.com/tiles/v4/{z}/{x}/{y}.mvt?key=YOUR_API_KEY",
  "flavor": "light",
  "lang": "en",
  "maxDataZoom": 11,
  "maxZoom": 22
}
```

**Local `.pmtiles` file (offline):**

```json
{
  "url": "netherlands_z11.pmtiles",
  "flavor": "light",
  "lang": "nl",
  "maxDataZoom": 11,
  "maxZoom": 22,
  "bounds": [
    { "lat": 50.625073, "lng": 3.175049 },
    { "lat": 53.644638, "lng": 7.459717 }
  ]
}
```

TODO: bounds are not always respected by protomaps, when they are specified like this

### Vector Pairs

The pairs of grib parameters that are considered vector pairs are specified in the `settings/vectorPairs.json`.