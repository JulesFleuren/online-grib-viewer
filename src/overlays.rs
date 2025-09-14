use log::debug;
use wasm_bindgen::prelude::*;
use core::f64;
use std::fmt::{format, Write};


use crate::windbarbs::get_wind_barb_path;
use crate::projection::{epsg_3857_projection, inverse_epsg_3857_projection};

#[wasm_bindgen]
pub fn generate_wind_barbs_svg_overlay(
    n_lon: usize,
    n_lat: usize,
    step_size_lon: f64,
    step_size_lat: f64,
    u: Vec<f64>,
    v: Vec<f64>,
    zoom_level: i64,
) -> String {
    let cell_size = 250;
    let mut svg = String::new();

    // SVG header
    write!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" preserveAspectRatio="none"><style type="text/css">.svg-wb{{fill:#1A232D;stroke:#1A232D;stroke-width:3;stroke-linecap:round;stroke-linejoin:round;stroke-miterlimit:10;}}</style>"#,
        n_lon * cell_size,
        n_lat * cell_size
    ).unwrap();

    let (d_index, scale) = index_step_and_scale_based_on_zoom(step_size_lon, zoom_level);
    log::debug!("d_index: {}, scale: {}", d_index, scale);
    for i in (0..n_lat).step_by(d_index as usize) {
        for j in (0..n_lon).step_by(d_index as usize) {
            let idx = i * n_lon + j;
            let u_val = u[idx];
            let v_val = v[idx];
            if u_val.is_nan() || v_val.is_nan() { continue; }

            let magnitude = (u_val.powi(2) + v_val.powi(2)).sqrt();
            let direction = 90.0 - v_val.atan2(u_val) * 180.0 / std::f64::consts::PI;
            let translate = (
                (j as f64) * cell_size as f64,
                ((n_lat - i) as f64 - 1.0) * cell_size as f64
            );
            let barb_path = get_wind_barb_path(magnitude, 180.0 + direction, translate, 2.0 * scale);

            svg.push_str(&barb_path);
        }
    }

    svg.push_str("</svg>");
    svg
}

fn index_step_and_scale_based_on_zoom(step_size_lon: f64, zoom_level: i64) -> (i64, f64) {
    log::debug!("step_size_lon: {:.4}, zoom_level: {}", step_size_lon, zoom_level);
    // if the longitude spacing is 1 degree, we want max_zoom_level 5
    let max_zoom_level = 5 - f64::log2(step_size_lon) as i64;
    let index_step = 1_i64.max(2_f64.powf((max_zoom_level - zoom_level)as f64) as i64);
    let scale = index_step as f64;
    (index_step, scale)
}

#[wasm_bindgen]
pub fn generate_projected_wind_barbs_svg_overlay(
    lat: Vec<f64>,
    lon: Vec<f64>,
    n_lat: i64,
    n_lon: i64,
    u: Vec<f64>,
    v: Vec<f64>,
    zoom_level: i64,
) -> String {
    let cell_size = 250;
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

    let distance_between_arrows = f64::min(avg_dx, avg_dy);
    debug!("distance between arrows: {} {} {}", distance_between_arrows, avg_dx, avg_dy);
    let scale_multiplier = distance_between_arrows / 100.0;

    // SVG header
    write!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}" preserveAspectRatio="none"><style type="text/css">.svg-wb{{fill:#1A232D;stroke:#1A232D;stroke-width:3;stroke-linecap:round;stroke-linejoin:round;stroke-miterlimit:10;}}</style>"#,
        min_x,
        min_y,
        width,
        height,
    ).unwrap();

    let step_size_lon = 0.025;
    let (d_index, scale) = index_step_and_scale_based_on_zoom(step_size_lon, zoom_level);
    for i_lat in (0..n_lat).step_by(d_index as usize) {
        for i_lon in (0..n_lon).step_by(d_index as usize) {
            let idx = (i_lat*n_lon + i_lon) as usize;
            let u_val = u[idx];
            let v_val = v[idx];
            if u_val.is_nan() || v_val.is_nan() { continue; }

            let magnitude = (u_val.powi(2) + v_val.powi(2)).sqrt();
            let direction = 90.0 - v_val.atan2(u_val) * 180.0 / std::f64::consts::PI;

            let (x, y) = epsg_3857_projection(lat[idx], lon[idx]);

            let barb_path = get_wind_barb_path(magnitude, 180.0 + direction, (x, -y), scale_multiplier * scale);
            svg.push_str(&barb_path);
        }
    }
    svg.push_str("</svg>");
    svg
}
