use log::debug;
use wasm_bindgen::prelude::*;
use core::f64;
use std::fmt::{format, Write};

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
) -> String {
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
    svg
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