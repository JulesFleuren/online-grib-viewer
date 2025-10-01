use grib::{GribError, GridDefinition, GridDefinitionTemplateValues};
use log::debug;
use wasm_bindgen::prelude::*;
use core::f64;
use std::collections::HashMap;
use std::fmt::{format, Write};
use std::iter::Enumerate;

use crate::windbarbs::get_wind_barb_path;
use crate::projection::{epsg_3857_projection, inverse_epsg_3857_projection};

#[wasm_bindgen]
pub fn generate_wind_barbs_svg_overlay(
    lat: Vec<f64>,
    lon: Vec<f64>,
    n_lat: i64,
    n_lon: i64,
    u: Vec<f64>,
    v: Vec<f64>,
    zoom_level: i64,
) -> Result<JsValue, JsValue> {
    let mut svg = String::new();

    let min_lat = lat.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_lat = lat.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_lon = lon.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_lon = lon.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let (mut min_x, mut min_y) = epsg_3857_projection(min_lat, min_lon);
    let (mut max_x, mut max_y) = epsg_3857_projection(max_lat, max_lon);
    let mut width = max_x - min_x;
    let mut height = max_y - min_y;

    // reverse y because origin of svg coordinate system is at top left instead of bottom left
    (min_y, max_y) = (-max_y, -min_y);

    let avg_dx = width / n_lon as f64;
    let avg_dy = height / n_lat as f64;

    // To ensure that the border barbs are not cut in half we add a border of half a cell around the outside
    min_y -= avg_dy / 2.0;
    max_y += avg_dy / 2.0;
    min_x -= avg_dx / 2.0;
    max_x += avg_dx / 2.0;
    width = max_x - min_x;
    height = max_y - min_y;

    
    // SVG header
    write!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}" preserveAspectRatio="none"><style type="text/css">.svg-wb{{fill:#1A232D;stroke:#1A232D;stroke-width:3;stroke-linecap:round;stroke-linejoin:round;stroke-miterlimit:10;}}</style>"#,
        min_x,
        min_y,
        width,
        height,
    ).unwrap();
    
    // TODO: handle grids that are ordered column major
    let (index_step, scale) = index_step_and_scale_based_on_zoom(avg_dx, avg_dy, zoom_level);
    for i_lat in (0..n_lat).step_by(index_step) {
        for i_lon in (0..n_lon).step_by(index_step) {
            let idx = (i_lat*n_lon + i_lon) as usize;
            let u_val = u[idx];
            let v_val = v[idx];
            if u_val.is_nan() || v_val.is_nan() { continue; }

            let magnitude = (u_val.powi(2) + v_val.powi(2)).sqrt();
            let direction = 90.0 - v_val.atan2(u_val) * 180.0 / std::f64::consts::PI;

            let (x, y) = epsg_3857_projection(lat[idx], lon[idx]);

            let barb_path = get_wind_barb_path(magnitude, 180.0 + direction, (x, -y), scale);
            svg.push_str(&barb_path);
        }
    }
    svg.push_str("</svg>");
    
    let return_value = js_sys::Object::new();
    js_sys::Reflect::set(&return_value, &JsValue::from_str("svgString"), &JsValue::from(svg)).expect("failed to set svgString");

    Ok(JsValue::from(return_value))
}

fn index_step_and_scale_based_on_zoom(dx: f64, dy: f64, zoom_level: i64) -> (usize, f64) {
    let d_min = f64::min(dx, dy);

    // the numbers for max_zoom_level and scale_multiplier have been manually selected
    // max_zoom_level is the zoom level at which all barbs are shown
    // if d_min = 4096 we want max_zoom_level 10
    let max_zoom_level = 22 - f64::log2(d_min) as i64;
    // scale multiplier is the amount by which all barbs are scaled
    let scale_multiplier = d_min / 100.0;

    let index_step = usize::max(1_usize, 2_f64.powf((max_zoom_level - zoom_level)as f64) as usize);
    let scale = scale_multiplier*(index_step as f64);
    (index_step, scale)
}

fn get_lat_lon_1d(
    grid: &GridDefinitionTemplateValues,
) -> Result<(Vec<f32>, Vec<f32>), GribError> {
    match grid {
        GridDefinitionTemplateValues::Template0(grid) => {
            // TODO: extracting the 1d lats and lons is convoluted, but there doesn't seem to be an easy way to do it.
            // The RegularGridIterator has the two arrays we are looking for as fields, but they are private. Perhaps 
            // open an issue on grib-rs?

            let latlons = grid.latlons()?;
            let mut lat = vec![0_f32; grid.nj as usize];
            let mut lon = vec![0_f32; grid.ni as usize];
            let (lat_2d, lon_2d): (Vec<f32>, Vec<f32>) = latlons.unzip();
            for (idx, (i, j)) in grid.ij()?.enumerate() {
                debug!{"{}, {}", i, j};
                if i == 0 {
                    lat[j] = lat_2d[idx];
                }
                if j == 0 {
                    lon[i] = lon_2d[idx];
                }
            }
            return Ok((lat, lon));
        }
        GridDefinitionTemplateValues::Template20(_) => {
            // Polar stereographic logic here
            unimplemented!("1d lat/lon not implemented for Lambert grid")
        }
        GridDefinitionTemplateValues::Template30(_) => {
            // Lambert grid logic here
            unimplemented!("1d lat/lon not implemented for Lambert grid")
        }
        GridDefinitionTemplateValues::Template40(grid) => {
            let latlons = grid.latlons()?;
            let mut lat = vec![0_f32; grid.nj as usize];
            let mut lon = vec![0_f32; grid.ni as usize];
            let (lat_2d, lon_2d): (Vec<f32>, Vec<f32>) = latlons.unzip();
            for (i, j) in grid.ij()? {
                if i == 0 {
                    lat[j] = lat_2d[j];
                }
                if j == 0 {
                    lon[i] = lon_2d[i];
                }
            }
            return Ok((lat, lon));
        }
    }
}

fn get_index_map(grid: &GridDefinitionTemplateValues) -> Result<HashMap<(usize, usize), usize>, GribError> {
    // TODO: can this be done without a HashMap, but with a formula?
    let (ni, nj) = grid.grid_shape();
    let mut map: HashMap<(usize, usize), usize> = HashMap::with_capacity(ni * nj);
    for (idx, (i, j)) in grid.ij()?.enumerate() {
        map.insert((i, j), idx);
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use grib::GridDefinitionTemplateValues;

    use crate::overlays::{get_index_map, get_lat_lon_1d};


    #[test]
    fn test_get_lat_lon_1d() {
        let def = grib::LatLonGridDefinition {
            ni: 2,
            nj: 3,
            first_point_lat: 0,
            first_point_lon: 0,
            last_point_lat: 2_000_000,
            last_point_lon: 1_000_000,
            scanning_mode: grib::ScanningMode(0b01000000),
        };
        let grid = GridDefinitionTemplateValues::Template0(def);
        let (lat_1d, lon_1d) = get_lat_lon_1d(&grid).expect("get_lat_lon_1d failed");
        assert_eq!(lat_1d, vec![0.0, 1.0, 2.0]);
        assert_eq!(lon_1d, vec![0.0, 1.0]);
    }

    #[test]
    fn test_index_map() {
        let def = grib::LatLonGridDefinition {
            ni: 2,
            nj: 3,
            first_point_lat: 0,
            first_point_lon: 0,
            last_point_lat: 2_000_000,
            last_point_lon: 1_000_000,
            scanning_mode: grib::ScanningMode(0b01110000),
        };
        let grid = GridDefinitionTemplateValues::Template0(def);

        let ij = grid.ij().expect("ij failed");
        let map = get_index_map(&grid).expect("get_index_map failed");

        for (idx, (i, j)) in ij.enumerate() {
            assert_eq!(idx, *map.get(&(i, j)).expect(&format!("i: {}, j: {} not in map", i, j)));
        }
    }
}