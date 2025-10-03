use grib::{GribError, GridDefinitionTemplateValues};
use std::collections::HashMap;
use std::fmt::{Write};

use crate::windbarbs::get_wind_barb_path;
use crate::projection::{epsg_3857_projection};

pub struct SvgOverlay {
    pub svg_string: String,
    pub min_lat: f32,
    pub max_lat: f32,
    pub min_lon: f32,
    pub max_lon: f32,
    pub max_zoom_level: i64,
}

pub fn generate_wind_barbs_svg_overlay(
    grid: &GridDefinitionTemplateValues,
    u: Vec<f32>,
    v: Vec<f32>,
    zoom_level: i64,
) -> Result<SvgOverlay, GribError> {
    // TODO: check if u and v both have the size as the grid
    let (lat_1d, lon_1d) = get_lat_lon_1d(grid)?;
    let index_map = get_index_map(grid)?;

    let n_lat = lat_1d.len();
    let n_lon = lon_1d.len();


    // add border of half a cell around the edge
    // TODO: what if lat or lon only has 1 element?
    if lat_1d.len() == 1 || lon_1d.len() == 1 {
        todo!("lat or lon only has 1 element")
    }

    // corners of the overlay. The overlay extends half a cell beyond the corners of the grid.
    let min_lat = lat_1d[0] - (lat_1d[1] - lat_1d[0]) / 2.0_f32;
    let max_lat = lat_1d[lat_1d.len() - 1] + (lat_1d[lat_1d.len() - 1] - lat_1d[lat_1d.len() - 2]);
    let min_lon = lon_1d[0] - (lon_1d[1] - lon_1d[0]) / 2.0_f32;
    let max_lon = lon_1d[lon_1d.len() - 1] + (lon_1d[lon_1d.len() - 1] - lon_1d[lon_1d.len() - 2]);

    let (min_x_overlay, mut min_y_overlay) = epsg_3857_projection(min_lat, min_lon);
    let (max_x_overlay, mut max_y_overlay) = epsg_3857_projection(max_lat, max_lon);
    let width = max_x_overlay - min_x_overlay;
    let height = max_y_overlay - min_y_overlay;

    // reverse y because origin of svg coordinate system is at top left instead of bottom left
    (min_y_overlay, max_y_overlay) = (-max_y_overlay, -min_y_overlay);

    let avg_dx = width / (n_lon + 1) as f32;
    let avg_dy = height / (n_lat + 1) as f32;

    let mut svg_string = String::new();
    // SVG header
    write!(
        svg_string,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}" preserveAspectRatio="none"><style type="text/css">.svg-wb{{fill:#1A232D;stroke:#1A232D;stroke-width:3;stroke-linecap:round;stroke-linejoin:round;stroke-miterlimit:10;}}</style>"#,
        min_x_overlay,
        min_y_overlay,
        width,
        height,
    ).unwrap();


    let (index_step, scale, max_zoom_level) = index_step_and_scale_based_on_zoom(avg_dx, avg_dy, zoom_level);

    for i in (0..n_lon).step_by(index_step) {
        for j in (0..n_lat).step_by(index_step) {
            let idx = index_map.get(&(i, j)).unwrap();
            let u_val = u[*idx];
            let v_val = v[*idx];
            if u_val.is_nan() || v_val.is_nan() { continue; }

            let magnitude = (u_val.powi(2) + v_val.powi(2)).sqrt();
            let direction = 90.0 - v_val.atan2(u_val).to_degrees();

            let (x, y) = epsg_3857_projection(lat_1d[j], lon_1d[i]);

            let barb_path = get_wind_barb_path(magnitude, 180.0 + direction, (x, -y), scale);
            svg_string.push_str(&barb_path);
        }
    }
    svg_string.push_str("</svg>");


    Ok(SvgOverlay {svg_string, min_lat, max_lat, min_lon, max_lon, max_zoom_level})
}

fn index_step_and_scale_based_on_zoom(dx: f32, dy: f32, zoom_level: i64) -> (usize, f32, i64) {
    let d_min = f32::min(dx, dy);

    // the numbers for max_zoom_level and scale_multiplier have been manually selected
    // max_zoom_level is the zoom level at which all barbs are shown
    // if d_min = 4096 we want max_zoom_level 10
    let max_zoom_level = 22 - f32::log2(d_min) as i64;
    // scale multiplier is the amount by which all barbs are scaled
    let scale_multiplier = d_min / 100.0;

    let index_step = usize::max(1_usize, 2_f32.powf((max_zoom_level - zoom_level)as f32) as usize);
    let scale = scale_multiplier*(index_step as f32);
    (index_step, scale, max_zoom_level)
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
