use console_log::log;
use wasm_bindgen::prelude::*;
use console_error_panic_hook;
use grib::codetables::{Lookup, CodeTable4_2};
use js_sys::{Float32Array};
use std::{collections::HashMap, error::Error};
use std::collections::HashSet;

pub mod overlays;
pub mod windbarbs;

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("failed to initialize logging"); 
}

#[wasm_bindgen]
pub fn get_messages(bytes: &[u8]) -> Vec<JsValue> {
    let grib2 = grib::from_bytes(bytes).unwrap();
    let mut messages: Vec<JsValue> = Vec::new();
    for (index, message) in grib2.iter() {
        let discipline = message.indicator().discipline;
        let category = message.prod_def().parameter_category().expect("missing parameter category");
        let parameter = message.prod_def().parameter_number().expect("missing parameter number");

        let parameter_name = CodeTable4_2::new(discipline, category).lookup(usize::from(parameter));

        let temporal_info = grib::TemporalInfo::from(&message.temporal_raw_info());
        let forecast_time = temporal_info.forecast_time_target.unwrap().time();
        let forecast_date = temporal_info.forecast_time_target.unwrap().date_naive();

        // `fixed_surfaces()` returns a tuple of two layers wrapped by `Option`.
        let (first, _second) = message.prod_def().fixed_surfaces().expect("missing fixed surfaces");
        let elevation_level = first.value();
        let elevation_unit = first.unit().map(|s| format!(" [{s}]")).unwrap_or_default();
        
        let result_message = js_sys::Object::new();
        js_sys::Reflect::set(&result_message, &JsValue::from_str("index0"), &JsValue::from(index.0)).expect("failed to set index0");
        js_sys::Reflect::set(&result_message, &JsValue::from_str("index1"), &JsValue::from(index.1)).expect("failed to set index1");
        js_sys::Reflect::set(&result_message, &JsValue::from_str("name"), &JsValue::from(parameter_name.to_string())).expect("failed to set name");
        js_sys::Reflect::set(&result_message, &JsValue::from_str("category"), &JsValue::from(category.to_string())).expect("failed to set category");
        js_sys::Reflect::set(&result_message, &JsValue::from_str("forecast_time"), &JsValue::from(forecast_time.to_string())).expect("failed to set forecast_time");
        js_sys::Reflect::set(&result_message, &JsValue::from_str("forecast_date"), &JsValue::from(forecast_date.to_string())).expect("failed to set forecast_date");
        js_sys::Reflect::set(&result_message, &JsValue::from_str("elevation_level"), &JsValue::from(elevation_level.to_string())).expect("failed to set elevation_level");
        js_sys::Reflect::set(&result_message, &JsValue::from_str("elevation_unit"), &JsValue::from(elevation_unit.to_string())).expect("failed to set elevation_unit");
        messages.push(JsValue::from(result_message));
    }
    messages
}

#[wasm_bindgen]
pub fn get_grid_shape(bytes: &[u8], message_index: usize, message_subindex: usize) -> JsValue {
    let grib2 = grib::from_bytes(bytes).unwrap();
    // Find the target submessage.
    let (_index, submessage) = grib2
        .iter()
        .find(|(index, _)| *index == (message_index, message_subindex))
        .ok_or("no such index").unwrap();
    let (grid_shape_nx, grid_shape_ny) = submessage.grid_shape().unwrap();
    let result = js_sys::Object::new();
    js_sys::Reflect::set(&result, &JsValue::from_str("nx"), &JsValue::from(grid_shape_nx)).unwrap();
    js_sys::Reflect::set(&result, &JsValue::from_str("ny"), &JsValue::from(grid_shape_ny)).unwrap();
    JsValue::from(result)
}   

#[wasm_bindgen]
pub fn get_available_parameters(bytes: &[u8]) -> Vec<JsValue> {
    let grib2 = grib::from_bytes(bytes).unwrap();
    let mut parameters: HashMap<String, (u8, u8, u8)> = HashMap::new();
    for (_, message) in grib2.iter() {
        let discipline_number = message.indicator().discipline;
        let category_number = message.prod_def().parameter_category().expect("missing parameter category");
        let parameter_number = message.prod_def().parameter_number().expect("missing parameter number");

        let parameter_name = CodeTable4_2::new(discipline_number, category_number).lookup(usize::from(parameter_number));

        parameters.entry(parameter_name.to_string()).or_insert((discipline_number, category_number, parameter_number));
    }

    let mut js_parameters: Vec<JsValue> = Vec::new();
    for (parameter_name, (discipline_number, category_number, parameter_number)) in parameters {
        let parameter = js_sys::Object::new();
        js_sys::Reflect::set(&parameter, &JsValue::from_str("name"), &JsValue::from(parameter_name.to_string())).expect("failed to set name");
        js_sys::Reflect::set(&parameter, &JsValue::from_str("key"), &JsValue::from(format!("grib2_{}_{}_{}", discipline_number, category_number, parameter_number))).expect("failed to set key");
        js_parameters.push(JsValue::from(parameter));
    }
    js_parameters
}

#[wasm_bindgen]
pub fn get_available_timestamps(bytes: &[u8], key: &str) -> Vec<JsValue> {
    let grib2 = grib::from_bytes(bytes).unwrap();
    let mut times: HashSet<i64> = HashSet::new();
    for (_, message) in grib2.iter() {
        let parts: Vec<&str> = key.split('_').collect();
        if parts.len() != 4 || parts[0] != "grib2" {
            continue;
        }
        let discipline: u8 = parts[1].parse().expect("invalid discipline");
        let category: u8 = parts[2].parse().expect("invalid category");
        let parameter: u8 = parts[3].parse().expect("invalid parameter");
        let d = message.indicator().discipline;
        let c = message.prod_def().parameter_category().expect("missing parameter category");
        let p = message.prod_def().parameter_number().expect("missing parameter number");
        if d == discipline && c == category && p == parameter {
            let temporal_info = grib::TemporalInfo::from(&message.temporal_raw_info());
            let t = temporal_info.forecast_time_target.unwrap().timestamp();
            times.insert(t);
        }
    }

    // sort times
    let mut times: Vec<i64> = times.into_iter().collect();
    times.sort();

    let mut js_times: Vec<JsValue> = Vec::new();
    for time in times {
        js_times.push(JsValue::from(time));
    }
    js_times
}

#[wasm_bindgen]
pub fn query_grib_message_at_point(bytes: &[u8], key: &str, time: i64, query_lat: f32, query_lon: f32) -> Result<JsValue, JsValue> {
    let parts: Vec<&str> = key.split('_').collect();
    if parts.len() != 4 || parts[0] != "grib2" {
        return Err(JsValue::from("invalid key format"));
    }
    let discipline: u8 = parts[1].parse().expect("invalid discipline");
    let category: u8 = parts[2].parse().expect("invalid category");
    let parameter: u8 = parts[3].parse().expect("invalid parameter");

    let (index, subindex) = find_grib_index(bytes, discipline, category, parameter, time)
        .map_err(|e| JsValue::from(e))?;
    let (lat, lon, values) = decode_layer(bytes, (index, subindex)).expect("failed to decode layer");

    let nearest_point_index = find_closest_point_in_grid(&lat, &lon, query_lat, query_lon);

    let return_value = js_sys::Object::new();
    js_sys::Reflect::set(&return_value, &JsValue::from_str("lat"), &JsValue::from(lat[nearest_point_index])).expect("failed to set lat");
    js_sys::Reflect::set(&return_value, &JsValue::from_str("lon"), &JsValue::from(lon[nearest_point_index])).expect("failed to set lon");
    js_sys::Reflect::set(&return_value, &JsValue::from_str("value"), &JsValue::from(values[nearest_point_index])).expect("failed to set value");
    Ok(JsValue::from(return_value))
}

pub fn find_grib_index(bytes: &[u8], discipline: u8, category: u8, parameter: u8, time: i64) -> Result<(usize, usize), String> {
    let grib2 = grib::from_bytes(bytes).unwrap();
    for ((index, subindex), message) in grib2.iter() {
        let d = message.indicator().discipline;
        let c = message.prod_def().parameter_category().expect("missing parameter category");
        let p = message.prod_def().parameter_number().expect("missing parameter number");
        let temporal_info = grib::TemporalInfo::from(&message.temporal_raw_info());
        let t = temporal_info.forecast_time_target.unwrap().timestamp();
        if d == discipline && c == category && p == parameter && t == time {
            return Ok((index, subindex));
        }
    }
    Err("GRIB index not found".to_string())
}

#[wasm_bindgen]
pub fn get_scalar_field(bytes: &[u8], key: &str, time: i64) -> Result<JsValue, JsValue> {
    let parts: Vec<&str> = key.split('_').collect();
    if parts.len() != 4 || parts[0] != "grib2" {
        return Err(JsValue::from("invalid key format"));
    }
    let discipline: u8 = parts[1].parse().expect("invalid discipline");
    let category: u8 = parts[2].parse().expect("invalid category");
    let parameter: u8 = parts[3].parse().expect("invalid parameter");

    let (index, subindex) = find_grib_index(bytes, discipline, category, parameter, time)?;
    let (lat, lon, values) = decode_layer(bytes, (index, subindex)).expect("failed to decode layer");
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
pub fn get_vector_field(bytes: &[u8], u_index: usize, v_index: usize, u_subindex: usize, v_subindex: usize) -> JsValue {
    let (lat, lon, u) = decode_layer(bytes, (u_index, u_subindex)).unwrap();
    let (_, _, v) = decode_layer(bytes, (v_index, v_subindex)).unwrap();

    // Convert Rust Vec<f32> to JS Float32Array
    let lat = Float32Array::from(lat.as_slice());
    let lon = Float32Array::from(lon.as_slice());
    let u = Float32Array::from(u.as_slice());
    let v = Float32Array::from(v.as_slice());

    // Create a JS object with the arrays
    let result = js_sys::Object::new();
    js_sys::Reflect::set(&result, &JsValue::from_str("lat"), &lat).unwrap();
    js_sys::Reflect::set(&result, &JsValue::from_str("lon"), &lon).unwrap();
    js_sys::Reflect::set(&result, &JsValue::from_str("u"), &u).unwrap();
    js_sys::Reflect::set(&result, &JsValue::from_str("v"), &v).unwrap();

    JsValue::from(result)
}

fn find_closest_point_in_grid(lat: &Vec<f32>, lon: &Vec<f32>, query_lat: f32, query_lon: f32) -> usize {
    let mut closest_point_index = 0;
    let mut closest_distance = f32::MAX;
    for (i, (lat, lon)) in lat.iter().zip(lon.iter()).enumerate() {
        // Calculate the distance to the query point

        let distance = haversine_distance(*lat, *lon, query_lat, query_lon);
        // Update the closest point if this one is closer
        if distance < closest_distance {
            closest_distance = distance;
            closest_point_index = i;
        }
    }
    closest_point_index
}

fn decode_layer(byte_string: &[u8], message_index: (usize, usize)) -> Result<(Vec<f32>, Vec<f32>, Vec<f32>), Box<dyn Error>>
{
    // Parse the GRIB2 message.
    let grib2 = grib::from_bytes(byte_string)?;

    // Find the target submessage.
    let (_index, submessage) = grib2
        .iter()
        .find(|(index, _)| *index == message_index)
        .ok_or("no such index")?;

    // Obtain latitude-longitude locations as an iterator.
    let latlons = submessage.latlons()?;

    // create array with lats and lons
    let (lats, lons): (Vec<f32>, Vec<f32>) = latlons.unzip();

    // Prepare a decoder.
    let decoder = grib::Grib2SubmessageDecoder::from(submessage)?;

    // Actually dispatch a decoding process and get an iterator of decoded values.
    // There are various methods available for compressing GRIB2 data, but some are
    // not yet supported by this library and may return errors.
    let values_iterator = decoder.dispatch()?;

    // extract values from iterator
    let values = values_iterator
        .collect();


    Ok((lats, lons, values))
}

// Distance (meters) between two points (latitude and longitude in degrees)
fn haversine_distance(lat1: f32, lon1: f32, lat2: f32, lon2: f32) -> f32 {
    const EARTH_RADIUS: f32 = 6371008.7714;

    let a = ((lat2 - lat1) / 2.0).to_radians().sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * ((lon2 - lon1) / 2.0).to_radians().sin().powi(2);
    2.0 * EARTH_RADIUS * a.sqrt().asin()
}