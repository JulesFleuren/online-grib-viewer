# Online-Grib-Viewer

**Online-Grib-Viewer** is a tool for visualizing Grib2-files, right in your browser. It is hosted at [gribviewer.com](https://gribviewer.com).

The website runs completely client side. All data processing is done inside your browser, and no data is sent to a server. No software needs to be installed, except for a browser.

## Capabilities

Online-Grib-Viewer has the following capabilities:

- Display data on a slippy map, allowing zooming and panning around
- Selecting a Grib message based on parameter, timestamp and fixed surfaces
- Visualizing grib messages as a heatmap
- Visualizing grib messages as a vectorfield (with either barbs or arrows)
- Displaying basic data about a grib message
- Support for Latitude-Longitude grids and Gaussian grids

The following capabilities are not (yet) supported:

- Reading Grib1 files
- Reading Grib ensemble files
- Reading Grib messages on other grids than lat-lon or Gaussian
- ...

The website is still in early development. If you find any bugs, or have a feature request, don't hesitate to make an issue. Be sure to follow the issue templates.
