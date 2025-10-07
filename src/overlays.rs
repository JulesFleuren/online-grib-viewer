use colorgrad::Gradient;
use grib::{GribError, GridDefinitionTemplateValues};
use std::collections::HashMap;
use std::fmt::{Write};

use crate::windbarbs::{get_arrow_path, ArrowType};
use crate::projection::{epsg_3857_projection, inverse_epsg_3857_projection};

pub struct SvgOverlay {
    pub svg_string: String,
    pub min_lat: f32,
    pub max_lat: f32,
    pub min_lon: f32,
    pub max_lon: f32,
    pub max_zoom_level: i64,
}

pub struct ImageOverlay {
    pub image: Vec<u8>,
    pub width_px: usize,
    pub height_px: usize,
    pub min_lat: f32,
    pub max_lat: f32,
    pub min_lon: f32,
    pub max_lon: f32,
}

pub fn generate_vector_field_svg_overlay(
    grid: &GridDefinitionTemplateValues,
    u: Vec<f32>,
    v: Vec<f32>,
    zoom_level: i64,
    arrow_type: ArrowType,
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

            let barb_path = get_arrow_path(&arrow_type, magnitude, 180.0 + direction, (x, -y), scale);
            svg_string.push_str(&barb_path);
        }
    }
    svg_string.push_str("</svg>");


    Ok(SvgOverlay {svg_string, min_lat, max_lat, min_lon, max_lon, max_zoom_level})
}

pub fn generate_heatmap_overlay(
    grid: &GridDefinitionTemplateValues,
    values: Vec<f32>,
    pixels_per_cell: usize,
) -> Result<ImageOverlay, GribError> {
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

    let (min_x_overlay, min_y_overlay) = epsg_3857_projection(min_lat, min_lon);
    let (max_x_overlay, max_y_overlay) = epsg_3857_projection(max_lat, max_lon);

    let values_max = values.iter().filter(|&&x| !x.is_nan()).copied().reduce(f32::max).expect("values is empty");
    let values_min = values.iter().filter(|&&x| !x.is_nan()).copied().reduce(f32::min).unwrap();

    let width_px = n_lon * pixels_per_cell;
    let height_px = n_lat * pixels_per_cell;

    let width_single_pixel = (max_x_overlay - min_x_overlay) / (width_px as f32);
    let height_single_pixel = (max_y_overlay - min_y_overlay) / (height_px as f32);

    let mut image = Vec::with_capacity((width_px * height_px * 4) as usize);

    let color_gradient = colorgrad::preset::turbo();

    // variables used in loop
    let (mut i, mut j) = (0, 0);
    let (mut lat_0, mut lon_0, mut lat_1, mut lon_1) = (0_f32, 0_f32, 0_f32, 0_f32);
    for i_px in 0..height_px {
        for j_px in 0..width_px {
            let x = min_x_overlay + (j_px as f32) * width_single_pixel + width_single_pixel * 0.5;
            // y is reversed since images have origin at top left
            let y = max_y_overlay - (i_px as f32) * height_single_pixel - height_single_pixel * 0.5;
            let (lat, lon) = inverse_epsg_3857_projection(x, y);

            // if point is still in the same gridcell, reuse the gridpoints of last iteration, otherwise recalculate
            if lon_0 > lon || lon > lon_1 {
                i = first_bigger_than(&lon_1d, lon);
                lon_0 = lon_1d[i - 1];
                lon_1 = lon_1d[i];
            }
            if lat_0 > lat || lat > lat_1 {
                // lat_1d[j] is the smallest element that is bigger than lat
                j = first_bigger_than(&lat_1d, lat);
                lat_0 = lat_1d[j - 1];
                lat_1 = lat_1d[j];
            }

            let idx_00 = *index_map.get(&(i-1, j-1)).unwrap();
            let idx_01 = *index_map.get(&(i-1, j)).unwrap();
            let idx_10 = *index_map.get(&(i, j-1)).unwrap();
            let idx_11 = *index_map.get(&(i, j)).unwrap();

            // perform bilinear interpolation (in lat-lon space)
            let denominator = (lon_1 - lon_0) * (lat_1 - lat_0);
            let w_00 = (lon_1 - lon) * (lat_1 - lat) / denominator;
            let w_01 = (lon_1 - lon) * (lat - lat_0) / denominator;
            let w_10 = (lon - lon_0) * (lat_1 - lat) / denominator;
            let w_11 = (lon - lon_0) * (lat - lat_0) / denominator;

            // interpolated value
            let value = w_00 * values[idx_00] + w_01 * values[idx_01] + w_10 * values[idx_10] + w_11 * values[idx_11];
            let color = color_gradient.at((value - values_min)/(values_max-values_min)).to_rgba8();

            image.extend_from_slice(&color);
        }
    }

    Ok(ImageOverlay { image, width_px, height_px, min_lat, max_lat, min_lon, max_lon })
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

/// Finds i such that sorted_vec[i-1] <= target <= sorted_vec[i]. When target is smaller than all elements in
/// sorted_vec, it returns 1, when target is larger than all elements it returns sorted_vec.len() - 1. In this way,
/// sorted_vec[i-1] and sorted_vec[i] always exist.
fn first_bigger_than(sorted_vec: &[f32], target: f32) -> usize {
    let index = sorted_vec.binary_search_by(|x| x.partial_cmp(&target).unwrap());
    match index {
        Ok(i) => {
            // Perfect match: target is equal to sorted_vec[i]. Return i or 1 if i == 0
            if i > 0 {
                return i;
            } else {
                return 1_usize;
            }
        }
        Err(i) => {
            // If the target is not found, the first bigger element is at i.
            if i >= sorted_vec.len() {
                return i - 1;
            } else if i == 0 {
                return 1_usize;
            } else {
                return i;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use grib::GridDefinitionTemplateValues;

    use crate::overlays::{first_bigger_than, get_index_map, get_lat_lon_1d};


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

    #[test]
    fn test_first_bigger_than() {
        let array = vec![1.0f32, 2.0f32, 3.0f32, 4.0f32];
        assert_eq!(first_bigger_than(&array, 0.9f32), 1_usize);
        assert_eq!(first_bigger_than(&array, 1.0f32), 1_usize);
        assert_eq!(first_bigger_than(&array, 1.999f32), 1_usize);
        assert_eq!(first_bigger_than(&array, 2.5f32), 2_usize);
        assert_eq!(first_bigger_than(&array, 3.0f32), 2_usize);
        assert_eq!(first_bigger_than(&array, 3.5f32), 3_usize);
        assert_eq!(first_bigger_than(&array, 4.0f32), 3_usize);
        assert_eq!(first_bigger_than(&array, 4.1f32), 3_usize);
    }
}
