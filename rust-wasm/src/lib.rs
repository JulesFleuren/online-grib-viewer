use console_error_panic_hook;
use grib::codetables::{CodeTable4_2, Lookup};
use grib::{Grib2, SeekableGrib2Reader};
use js_sys::Float32Array;
use log::warn;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Cursor;
use std::iter::zip;
use wasm_bindgen::prelude::*;

use crate::error::*;
use crate::grib_helpers::*;
use crate::math::*;
use crate::overlays::*;

pub mod error;
pub mod grib_helpers;
pub mod math;
pub mod overlays;
pub mod projection;
pub mod windbarbs;

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("failed to initialize logging");
}

#[wasm_bindgen]
pub struct GribViewer {
    grib2: Grib2<SeekableGrib2Reader<Cursor<Vec<u8>>>>,
}

#[wasm_bindgen]
impl GribViewer {
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: Vec<u8>) -> Result<GribViewer, JsValue> {
        let grib2 = grib::from_bytes(bytes).map_err(|e| GribViewerError::from(e))?;
        Ok(Self { grib2 })
    }

    pub fn get_available_parameters(&self) -> Result<Vec<JsValue>, JsValue> {
        let mut parameters: HashMap<String, (u8, u8, u8)> = HashMap::new();
        for (_, message) in self.grib2.iter() {
            let discipline = message.indicator().discipline;
            let prod_def = message.prod_def();
            let Some(category) = prod_def.parameter_category() else {
                warn!(
                    "Unsupported product definition template number: {}, skipping message",
                    prod_def.prod_tmpl_num()
                );
                continue;
            };
            let parameter = prod_def
                .parameter_number()
                .expect("parameter_category() should have failed");

            let parameter_name =
                CodeTable4_2::new(discipline, category).lookup(usize::from(parameter));

            // insert parameter if it is not already present
            parameters
                .entry(parameter_name.to_string())
                .or_insert((discipline, category, parameter));
        }

        let mut js_parameters: Vec<JsValue> = Vec::new();
        for (parameter_name, (discipline_number, category_number, parameter_number)) in parameters {
            let parameter = js_sys::Object::new();
            js_sys::Reflect::set(
                &parameter,
                &JsValue::from_str("name"),
                &JsValue::from(parameter_name.to_string()),
            )
            .expect("failed to set name");
            js_sys::Reflect::set(
                &parameter,
                &JsValue::from_str("key"),
                &JsValue::from(format!(
                    "grib2_{}_{}_{}",
                    discipline_number, category_number, parameter_number
                )),
            )
            .expect("failed to set key");
            js_parameters.push(JsValue::from(parameter));
        }
        Ok(js_parameters)
    }

    pub fn get_available_surfaces(&self, key: &str) -> Result<Vec<JsValue>, JsValue> {
        let mut surfaces: HashMap<String, String> = HashMap::new();
        let (discipline, category, parameter) = grib_parameter_from_key(key)?;

        for (index, message) in
            iter_messages_of_parameter(&self.grib2, discipline, category, parameter)
        {
            if let Some(surfaces_info) = &message.prod_def().fixed_surfaces() {
                let (surface1, surface2) = surfaces_info;
                let surface_key = format!(
                    "surface_{}_{}_{}_{}_{}_{}",
                    surface1.surface_type,
                    surface1.scale_factor,
                    surface1.scaled_value,
                    surface2.surface_type,
                    surface2.scale_factor,
                    surface2.scaled_value
                );
                surfaces
                    .entry(surface_key)
                    .or_insert(format_surfaces(surface1, surface2));
            } else {
                warn!(
                    "Message with unsupported tamplate definition number, skipping message {:?}",
                    index
                );
                continue;
            }
        }

        let mut js_surfaces: Vec<JsValue> = Vec::new();

        // iterate over items sorted by key
        let mut surface_keys: Vec<_> = surfaces.keys().collect();
        surface_keys.sort(); // requires K: Ord

        for surface_key in surface_keys {
            let surface = js_sys::Object::new();
            js_sys::Reflect::set(
                &surface,
                &JsValue::from_str("key"),
                &JsValue::from(surface_key),
            )
            .expect("failed to set name");
            js_sys::Reflect::set(
                &surface,
                &JsValue::from_str("description"),
                &JsValue::from(surfaces[surface_key].to_string()),
            )
            .expect("failed to set key");
            js_surfaces.push(JsValue::from(surface));
        }
        Ok(js_surfaces)
    }

    pub fn get_available_timestamps(
        &self,
        parameter_key: &str,
        surface_key: &str,
    ) -> Result<Vec<JsValue>, JsValue> {
        let mut times: HashSet<i64> = HashSet::new();
        let (discipline, category, parameter) = grib_parameter_from_key(parameter_key)?;
        let (surface1, surface2) = fixed_surfaces_from_key(surface_key)?;

        for (index, message) in iter_messages_of_parameter_and_surface(
            &self.grib2,
            discipline,
            category,
            parameter,
            surface1,
            surface2,
        ) {
            let temporal_info = grib::TemporalInfo::from(&message.temporal_raw_info());
            if let Some(forecast_time) = temporal_info.forecast_time_target {
                times.insert(forecast_time.timestamp());
            } else {
                warn!(
                    "Message with invalid forecast time, skipping message {:?}",
                    index
                );
                continue;
            }
        }

        let mut sorted_times: Vec<i64> = times.into_iter().collect();
        sorted_times.sort();

        let js_times: Vec<JsValue> = sorted_times.into_iter().map(JsValue::from).collect();

        Ok(js_times)
    }

    pub fn query_grib_message_at_point(
        &self,
        parameter_key: &str,
        surface_key: &str,
        time: i64,
        query_lat: f32,
        query_lon: f32,
    ) -> Result<JsValue, JsValue> {
        let (discipline, category, parameter) = grib_parameter_from_key(parameter_key)?;
        let (surface1, surface2) = fixed_surfaces_from_key(surface_key)?;

        let message = get_message(
            &self.grib2,
            discipline,
            category,
            parameter,
            &surface1,
            &surface2,
            time,
        )?;
        let (lat, lon, values) = get_lat_lon_and_values(message)?;

        let nearest_point_index = find_closest_point_in_grid(&lat, &lon, query_lat, query_lon);

        let return_value = js_sys::Object::new();
        js_sys::Reflect::set(
            &return_value,
            &JsValue::from_str("lat"),
            &JsValue::from(lat[nearest_point_index]),
        )
        .expect("failed to set lat");
        js_sys::Reflect::set(
            &return_value,
            &JsValue::from_str("lon"),
            &JsValue::from(lon[nearest_point_index]),
        )
        .expect("failed to set lon");
        js_sys::Reflect::set(
            &return_value,
            &JsValue::from_str("value"),
            &JsValue::from(values[nearest_point_index]),
        )
        .expect("failed to set value");
        Ok(JsValue::from(return_value))
    }
}

#[wasm_bindgen]
pub fn get_scalar_field(
    bytes: &[u8],
    parameter_key: &str,
    surface_key: &str,
    time: i64,
) -> Result<JsValue, JsValue> {
    let (discipline, category, parameter) = grib_parameter_from_key(parameter_key)?;
    let (surface1, surface2) = fixed_surfaces_from_key(surface_key)?;

    let grib2 = grib::from_bytes(bytes).map_err(|e| JsValue::from(GribViewerError::from(e)))?;

    let message = get_message(
        &grib2, discipline, category, parameter, &surface1, &surface2, time,
    )?;

    let (lat, lon, values) = get_lat_lon_and_values(message)?;
    // Convert Rust Vec<f32> to JS Float32Array
    let lat = Float32Array::from(lat.as_slice());
    let lon = Float32Array::from(lon.as_slice());
    let values = Float32Array::from(values.as_slice());

    // Create a JS object with the arrays
    let result = js_sys::Object::new();
    js_sys::Reflect::set(&result, &JsValue::from_str("lat"), &lat).expect("failed to set lat");
    js_sys::Reflect::set(&result, &JsValue::from_str("lon"), &lon).expect("failed to set lon");
    js_sys::Reflect::set(&result, &JsValue::from_str("values"), &values)
        .expect("failed to set values");

    Ok(JsValue::from(result))
}

#[wasm_bindgen]
pub fn vector_field_overlay(
    bytes: &[u8],
    param_key_u: &str,
    param_key_v: &str,
    surface_key: &str,
    time: i64,
    zoom_level: i64,
    settings: JsValue,
) -> Result<JsValue, JsValue> {
    let settings = serde_wasm_bindgen::from_value(settings)
        .map_err(|e| GribViewerError::Other(format!("Error deserializing settings: {}", e)))?;

    let (surface1, surface2) = fixed_surfaces_from_key(surface_key)?;

    let grib2 = grib::from_bytes(bytes).map_err(|e| JsValue::from(GribViewerError::from(e)))?;

    let param_u = grib_parameter_from_key(param_key_u)?;
    // This structure is to prevent a panic about reborrowing. In this way message_u lives as short as possible
    let (grid, u) = {
        let message_u = get_message(
            &grib2, param_u.0, param_u.1, param_u.2, &surface1, &surface2, time,
        )?;
        get_grid_and_values(message_u)?
    };

    // it is assumed that u and v have the same grid
    // TODO: should this be checked?
    let param_v = grib_parameter_from_key(param_key_v)?;
    let (_grid, v) = {
        let message_v = get_message(
            &grib2, param_v.0, param_v.1, param_v.2, &surface1, &surface2, time,
        )?;
        get_grid_and_values(message_v)?
    };

    let svg_overlay = generate_vector_field_svg_overlay(&grid, u, v, zoom_level, settings)?;

    Ok(serde_wasm_bindgen::to_value(&svg_overlay)?)
}

#[wasm_bindgen]
pub fn heatmap_overlay(
    bytes: &[u8],
    param_key: &str,
    surface_key: &str,
    time: i64,
    settings: JsValue,
) -> Result<JsValue, JsValue> {
    let settings = serde_wasm_bindgen::from_value(settings)
        .map_err(|e| GribViewerError::Other(format!("Error deserializing settings: {}", e)))?;

    let (discipline, category, parameter) = grib_parameter_from_key(param_key)?;
    let (surface1, surface2) = fixed_surfaces_from_key(surface_key)?;

    let grib2 = grib::from_bytes(bytes).map_err(|e| JsValue::from(GribViewerError::from(e)))?;

    // This structure is to prevent a panic about reborrowing. In this way message lives as short as possible
    let (grid, values) = {
        let message = get_message(
            &grib2, discipline, category, parameter, &surface1, &surface2, time,
        )?;

        get_grid_and_values(message)?
    };

    let image_overlay = generate_heatmap_overlay(&grid, values, settings)?;

    Ok(serde_wasm_bindgen::to_value(&image_overlay)?)
}

#[wasm_bindgen]
pub fn magnitude_heatmap_overlay(
    bytes: &[u8],
    param_key_u: &str,
    param_key_v: &str,
    surface_key: &str,
    time: i64,
    heatmap_overlay_settings: JsValue,
) -> Result<JsValue, JsValue> {
    let heatmap_overlay_settings = serde_wasm_bindgen::from_value(heatmap_overlay_settings)
        .map_err(|e| GribViewerError::Other(format!("Error deserializing settings: {}", e)))?;

    let (surface1, surface2) = fixed_surfaces_from_key(surface_key)?;

    let grib2 = grib::from_bytes(bytes).map_err(|e| JsValue::from(GribViewerError::from(e)))?;

    let param_u = grib_parameter_from_key(param_key_u)?;
    // This structure is to prevent a panic about reborrowing. In this way message_u lives as short as possible
    let (grid, u) = {
        let message_u = get_message(
            &grib2, param_u.0, param_u.1, param_u.2, &surface1, &surface2, time,
        )?;
        get_grid_and_values(message_u)?
    };

    // it is assumed that u and v have the same grid
    // TODO: should this be checked?
    let param_v = grib_parameter_from_key(param_key_v)?;
    let (_grid, v) = {
        let message_v = get_message(
            &grib2, param_v.0, param_v.1, param_v.2, &surface1, &surface2, time,
        )?;
        get_grid_and_values(message_v)?
    };

    let values = norm(&u, &v);

    let image_overlay = generate_heatmap_overlay(&grid, values, heatmap_overlay_settings)?;

    Ok(serde_wasm_bindgen::to_value(&image_overlay)?)
}

/// Go through all grib messages with this key, and find the minimum and maximum occuring value.
#[wasm_bindgen]
pub fn find_min_max_value(bytes: &[u8], key: &str) -> Result<JsValue, JsValue> {
    let grib2 = grib::from_bytes(bytes).map_err(|e| GribViewerError::from(e))?;
    let (discipline, category, parameter) = grib_parameter_from_key(key)?;

    let (mut param_min, mut param_max) = (f32::INFINITY, f32::NEG_INFINITY);
    for (index, message) in iter_messages_of_parameter(&grib2, discipline, category, parameter) {
        let decoder =
            grib::Grib2SubmessageDecoder::from(message).map_err(|e| GribViewerError::from(e))?;
        let mut values_iter = decoder.dispatch().map_err(|e| GribViewerError::from(e))?;

        let first = match values_iter.next() {
            Some(val) => (val, val),
            None => {
                warn!("Grib message {:?} is empty, skipping message", index);
                continue;
            }
        };

        // iterate over all values in message and find the min and max value (skipping Nan's)
        let (message_min, message_max) =
            values_iter.fold(first, |(min, max), val| (min.min(val), max.max(val)));
        param_min = param_min.min(message_min);
        param_max = param_max.max(message_max);
    }

    if !(param_min.is_finite() && param_max.is_finite()) {
        return Err(GribViewerError::Other(
            "No matching grib messages found, or all messages were emtpy".to_string(),
        )
        .into());
    }

    let result = js_sys::Object::new();
    js_sys::Reflect::set(
        &result,
        &JsValue::from_str("min"),
        &JsValue::from(param_min),
    )
    .expect("failed to set min");
    js_sys::Reflect::set(
        &result,
        &JsValue::from_str("max"),
        &JsValue::from(param_max),
    )
    .expect("failed to set max");
    Ok(JsValue::from(result))
}

/// Go through all grib messages with this key, and find the minimum and maximum occuring value.
#[wasm_bindgen]
pub fn find_min_max_magnitude(bytes: &[u8], key_u: &str, key_v: &str) -> Result<JsValue, JsValue> {
    let grib2 = grib::from_bytes(bytes).map_err(|e| GribViewerError::from(e))?;
    let param_u = grib_parameter_from_key(key_u)?;
    let param_v = grib_parameter_from_key(key_v)?;

    let (mut param_min, mut param_max) = (f32::INFINITY, f32::NEG_INFINITY);
    'u: for (_, message_u) in iter_messages_of_parameter(&grib2, param_u.0, param_u.1, param_u.2) {
        let temporal_info = grib::TemporalInfo::from(&message_u.temporal_raw_info());
        let Some(t_u) = temporal_info.forecast_time_target else {
            continue;
        };

        let decoder_u =
            grib::Grib2SubmessageDecoder::from(message_u).map_err(|e| GribViewerError::from(e))?;
        let u_iter = decoder_u.dispatch().map_err(|e| GribViewerError::from(e))?;

        // find the v component with matching timestamp
        'v: for (_, message_v) in
            iter_messages_of_parameter(&grib2, param_v.0, param_v.1, param_v.2)
        {
            let temporal_info = grib::TemporalInfo::from(&message_v.temporal_raw_info());
            let Some(t_v) = temporal_info.forecast_time_target else {
                continue 'v;
            };

            if t_u == t_v {
                // If we have found a message_u and a message_v with matching timestamps, construct iter over v values
                let decoder_v = grib::Grib2SubmessageDecoder::from(message_v)
                    .map_err(|e| GribViewerError::from(e))?;
                let v_iter = decoder_v.dispatch().map_err(|e| GribViewerError::from(e))?;

                // iterator over both components
                let mut iter = zip(u_iter, v_iter);

                // iterate over both components, calculate their norm and find min and max of norm
                if let Some((first_u, first_v)) = iter.next() {
                    let first_norm = (first_u * first_u + first_v * first_v).sqrt();
                    let (message_min, message_max) =
                        iter.fold((first_norm, first_norm), |(min, max), (u, v)| {
                            let norm = (u * u + v * v).sqrt();
                            (min.min(norm), max.max(norm))
                        });
                    param_min = param_min.min(message_min);
                    param_max = param_max.max(message_max);
                }
                // min, max has been updated, continue outer loop
                continue 'u;
            }
        }
    }

    if !(param_min.is_finite() && param_max.is_finite()) {
        return Err(GribViewerError::Other(
            "No matching grib messages found, or all messages were emtpy".to_string(),
        )
        .into());
    }

    let result = js_sys::Object::new();
    js_sys::Reflect::set(
        &result,
        &JsValue::from_str("min"),
        &JsValue::from(param_min),
    )
    .expect("failed to set min");
    js_sys::Reflect::set(
        &result,
        &JsValue::from_str("max"),
        &JsValue::from(param_max),
    )
    .expect("failed to set max");
    Ok(JsValue::from(result))
}

#[wasm_bindgen]
pub fn get_message_info(
    bytes: &[u8],
    parameter_key: &str,
    surface_key: &str,
    time: i64,
) -> Result<JsValue, JsValue> {
    let (discipline, category, parameter) = grib_parameter_from_key(parameter_key)?;
    let (surface1, surface2) = fixed_surfaces_from_key(surface_key)?;

    let grib2 = grib::from_bytes(bytes).map_err(|e| JsValue::from(GribViewerError::from(e)))?;

    let message = get_message(
        &grib2, discipline, category, parameter, &surface1, &surface2, time,
    )?;

    let output = message.describe();
    Ok(JsValue::from(output))
}

#[wasm_bindgen]
pub fn get_message_dump(
    bytes: &[u8],
    parameter_key: &str,
    surface_key: &str,
    time: i64,
) -> Result<JsValue, JsValue> {
    let (discipline, category, parameter) = grib_parameter_from_key(parameter_key)?;
    let (surface1, surface2) = fixed_surfaces_from_key(surface_key)?;

    let grib2 = grib::from_bytes(bytes).map_err(|e| JsValue::from(GribViewerError::from(e)))?;

    let message = get_message(
        &grib2, discipline, category, parameter, &surface1, &surface2, time,
    )?;

    let mut buffer = Vec::new();

    message
        .dump(&mut buffer)
        .map_err(|e| GribViewerError::from(e))?;

    Ok(JsValue::from(
        String::from_utf8(buffer).map_err(|e| GribViewerError::Other(e.to_string()))?,
    ))
}
