use log::warn;
use wasm_bindgen::prelude::*;
use console_error_panic_hook;
use grib::codetables::{Lookup, CodeTable4_2};
use js_sys::{Float32Array};
use std::{collections::HashMap};
use std::collections::HashSet;

use crate::error::*;
use crate::overlays::*;
use crate::windbarbs::*;
use crate::grib_helpers::*;
use crate::math::*;

pub mod overlays;
pub mod windbarbs;
pub mod projection;
pub mod error;
pub mod grib_helpers;
pub mod math;

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("failed to initialize logging");
}

#[wasm_bindgen]
pub fn get_available_parameters(bytes: &[u8]) -> Result<Vec<JsValue>, JsValue> {
    let grib2 = grib::from_bytes(bytes).map_err(|e| GribViewerError::from(e))?;
    let mut parameters: HashMap<String, (u8, u8, u8)> = HashMap::new();
    for (_, message) in grib2.iter() {
        let discipline = message.indicator().discipline;
        let prod_def = message.prod_def();
        let Some(category) = prod_def.parameter_category() else {
            warn!("Unsupported product definition template number: {}, skipping message", prod_def.prod_tmpl_num());
            continue;
            };
        let parameter = prod_def.parameter_number().expect("parameter_category() should have failed");

        let parameter_name = CodeTable4_2::new(discipline, category).lookup(usize::from(parameter));

        // insert parameter if it is not already present
        parameters.entry(parameter_name.to_string()).or_insert((discipline, category, parameter));
    }

    let mut js_parameters: Vec<JsValue> = Vec::new();
    for (parameter_name, (discipline_number, category_number, parameter_number)) in parameters {
        let parameter = js_sys::Object::new();
        js_sys::Reflect::set(&parameter, &JsValue::from_str("name"), &JsValue::from(parameter_name.to_string())).expect("failed to set name");
        js_sys::Reflect::set(
            &parameter, &JsValue::from_str("key"),
            &JsValue::from(format!("grib2_{}_{}_{}", discipline_number, category_number, parameter_number))
        ).expect("failed to set key");
        js_parameters.push(JsValue::from(parameter));
    }
    Ok(js_parameters)
}

#[wasm_bindgen]
pub fn get_available_timestamps(bytes: &[u8], key: &str) -> Result<Vec<JsValue>, JsValue> {
    let grib2 = grib::from_bytes(bytes)
        .map_err(|e| GribViewerError::from(e))?;

    let mut times: HashSet<i64> = HashSet::new();
    let (discipline, category, parameter) = grib_parameter_from_key(key)?;

    for (index, message) in grib2.iter() {
        let d = message.indicator().discipline;
        let prod_def = message.prod_def();
        let Some(c) = prod_def.parameter_category() else {
            warn!("Unsupported product definition template number: {}, skipping message {:?}", prod_def.prod_tmpl_num(), index);
            continue;
            };
        let p = prod_def.parameter_number().expect("parameter_category() should have failed");

        if d == discipline && c == category && p == parameter {
            let temporal_info = grib::TemporalInfo::from(&message.temporal_raw_info());
            if let Some(forecast_time) = temporal_info.forecast_time_target {
                times.insert(forecast_time.timestamp());
            } else {
                warn!("Message with invalid forecast time, skipping message {:?}", index);
                continue;
            }
        }
    }

    let mut sorted_times: Vec<i64> = times.into_iter().collect();
    sorted_times.sort();

    let js_times: Vec<JsValue> = sorted_times
        .into_iter()
        .map(JsValue::from)
        .collect();

    Ok(js_times)
}

#[wasm_bindgen]
pub fn query_grib_message_at_point(bytes: &[u8], key: &str, time: i64, query_lat: f32, query_lon: f32) -> Result<JsValue, JsValue> {
    let parts: Vec<&str> = key.split('_').collect();
    if parts.len() != 4 || parts[0] != "grib2" {
        return Err(JsValue::from_str("Invalid key format. Expected 'grib2_<disc>_<cat>_<param>'"));
    }

    let discipline: u8 = parts[1]
        .parse()
        .map_err(|_| JsValue::from_str("Invalid discipline in key"))?;
    let category: u8 = parts[2]
        .parse()
        .map_err(|_| JsValue::from_str("Invalid category in key"))?;
    let parameter: u8 = parts[3]
        .parse()
        .map_err(|_| JsValue::from_str("Invalid parameter in key"))?;

    let (index, subindex) = find_grib_index(bytes, discipline, category, parameter, time)
        .map_err(|e| JsValue::from(e))?;
    let (lat, lon, values) = get_lat_lon_and_values(bytes, (index, subindex))?;

    let nearest_point_index = find_closest_point_in_grid(&lat, &lon, query_lat, query_lon);

    let return_value = js_sys::Object::new();
    js_sys::Reflect::set(&return_value, &JsValue::from_str("lat"), &JsValue::from(lat[nearest_point_index])).expect("failed to set lat");
    js_sys::Reflect::set(&return_value, &JsValue::from_str("lon"), &JsValue::from(lon[nearest_point_index])).expect("failed to set lon");
    js_sys::Reflect::set(&return_value, &JsValue::from_str("value"), &JsValue::from(values[nearest_point_index])).expect("failed to set value");
    Ok(JsValue::from(return_value))
}

#[wasm_bindgen]
pub fn get_scalar_field(bytes: &[u8], key: &str, time: i64) -> Result<JsValue, JsValue> {
    let (discipline, category, parameter) = grib_parameter_from_key(key)?;

    let (index, subindex) = find_grib_index(bytes, discipline, category, parameter, time)?;
    let (lat, lon, values) = get_lat_lon_and_values(bytes, (index, subindex))?;
    // Convert Rust Vec<f32> to JS Float32Array
    let lat = Float32Array::from(lat.as_slice());
    let lon = Float32Array::from(lon.as_slice());
    let values = Float32Array::from(values.as_slice());

    // Create a JS object with the arrays
    let result = js_sys::Object::new();
    js_sys::Reflect::set(&result, &JsValue::from_str("lat"), &lat).expect("failed to set lat");
    js_sys::Reflect::set(&result, &JsValue::from_str("lon"), &lon).expect("failed to set lon");
    js_sys::Reflect::set(&result, &JsValue::from_str("values"), &values).expect("failed to set values");

    Ok(JsValue::from(result))
}

#[wasm_bindgen]
pub fn vector_field_overlay(bytes: &[u8], key_u: &str, key_v: &str, time: i64, zoom_level: i64, arrow_type: &str) -> Result<JsValue, JsValue> {
    let arrow_type: ArrowType = serde_plain::from_str(arrow_type)
        .map_err(|e| GribViewerError::Other(format!("Invalid arrow type: {}", e)))?;

    let param_u = grib_parameter_from_key(key_u)?;
    let index_u = find_grib_index(bytes, param_u.0, param_u.1, param_u.2, time)?;
    let param_v = grib_parameter_from_key(key_v)?;
    let index_v = find_grib_index(bytes, param_v.0, param_v.1, param_v.2, time)?;

    // it is assumed that u and v have the same grid
    // // TODO: should this be checked?
    let (grid, u) = get_grid_and_values(bytes, index_u)?;
    let (_grid, v) = get_grid_and_values(bytes, index_v)?;

    let svg_overlay = generate_vector_field_svg_overlay(&grid, u, v, zoom_level, arrow_type)?;

    // Create a JS object with the arrays
    let result = js_sys::Object::new();
    js_sys::Reflect::set(&result, &JsValue::from_str("svgString"), &JsValue::from(svg_overlay.svg_string)).expect("failed to set svgString");
    js_sys::Reflect::set(&result, &JsValue::from_str("minLat"), &JsValue::from(svg_overlay.min_lat)).expect("failed to set minLat");
    js_sys::Reflect::set(&result, &JsValue::from_str("maxLat"), &JsValue::from(svg_overlay.max_lat)).expect("failed to set maxLat");
    js_sys::Reflect::set(&result, &JsValue::from_str("minLon"), &JsValue::from(svg_overlay.min_lon)).expect("failed to set minLon");
    js_sys::Reflect::set(&result, &JsValue::from_str("maxLon"), &JsValue::from(svg_overlay.max_lon)).expect("failed to set maxLon");
    js_sys::Reflect::set(&result, &JsValue::from_str("maxZoomLevel"), &JsValue::from(svg_overlay.max_zoom_level)).expect("failed to set maxZoomLevel");

    Ok(JsValue::from(result))
}

#[wasm_bindgen]
pub fn heatmap_overlay(bytes: &[u8], key: &str, time: i64) -> Result<JsValue, JsValue> {
    let pixels_per_cell = 3;
    let param = grib_parameter_from_key(key)?;
    let index = find_grib_index(bytes, param.0, param.1, param.2, time)?;

    let (grid, values) = get_grid_and_values(bytes, index)?;

    let image_overlay = generate_heatmap_overlay(&grid, values, pixels_per_cell)?;

    // Create a JS object with the arrays
    let result = js_sys::Object::new();
    js_sys::Reflect::set(&result, &JsValue::from_str("image"), &JsValue::from(image_overlay.image)).expect("failed to set image");
    js_sys::Reflect::set(&result, &JsValue::from_str("width"), &JsValue::from(image_overlay.width_px)).expect("failed to set width");
    js_sys::Reflect::set(&result, &JsValue::from_str("height"), &JsValue::from(image_overlay.height_px)).expect("failed to set height");
    js_sys::Reflect::set(&result, &JsValue::from_str("minLat"), &JsValue::from(image_overlay.min_lat)).expect("failed to set minLat");
    js_sys::Reflect::set(&result, &JsValue::from_str("maxLat"), &JsValue::from(image_overlay.max_lat)).expect("failed to set maxLat");
    js_sys::Reflect::set(&result, &JsValue::from_str("minLon"), &JsValue::from(image_overlay.min_lon)).expect("failed to set minLon");
    js_sys::Reflect::set(&result, &JsValue::from_str("maxLon"), &JsValue::from(image_overlay.max_lon)).expect("failed to set maxLon");

    Ok(JsValue::from(result))
}

#[wasm_bindgen]
pub fn magnitude_heatmap_overlay(bytes: &[u8], key_u: &str, key_v: &str, time: i64) -> Result<JsValue, JsValue> {
    let pixels_per_cell = 3;
    let param_u = grib_parameter_from_key(key_u)?;
    let index_u = find_grib_index(bytes, param_u.0, param_u.1, param_u.2, time)?;
    let param_v = grib_parameter_from_key(key_v)?;
    let index_v = find_grib_index(bytes, param_v.0, param_v.1, param_v.2, time)?;

    // it is assumed that u and v have the same grid
    // TODO: should this be checked?
    let (grid, u) = get_grid_and_values(bytes, index_u)?;
    let (_grid, v) = get_grid_and_values(bytes, index_v)?;

    let values = norm(&u, &v);

    let image_overlay = generate_heatmap_overlay(&grid, values, pixels_per_cell)?;

    // Create a JS object with the arrays
    let result = js_sys::Object::new();
    js_sys::Reflect::set(&result, &JsValue::from_str("image"), &JsValue::from(image_overlay.image)).expect("failed to set image");
    js_sys::Reflect::set(&result, &JsValue::from_str("width"), &JsValue::from(image_overlay.width_px)).expect("failed to set width");
    js_sys::Reflect::set(&result, &JsValue::from_str("height"), &JsValue::from(image_overlay.height_px)).expect("failed to set height");
    js_sys::Reflect::set(&result, &JsValue::from_str("minLat"), &JsValue::from(image_overlay.min_lat)).expect("failed to set minLat");
    js_sys::Reflect::set(&result, &JsValue::from_str("maxLat"), &JsValue::from(image_overlay.max_lat)).expect("failed to set maxLat");
    js_sys::Reflect::set(&result, &JsValue::from_str("minLon"), &JsValue::from(image_overlay.min_lon)).expect("failed to set minLon");
    js_sys::Reflect::set(&result, &JsValue::from_str("maxLon"), &JsValue::from(image_overlay.max_lon)).expect("failed to set maxLon");

    Ok(JsValue::from(result))
}
