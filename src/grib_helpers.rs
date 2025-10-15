use grib::{Grib2, GridDefinitionTemplateValues, MessageIndex, SubMessage};
use log::warn;
use std::io::Read;

use crate::error::GribViewerError;

/// Filter grib.iter() so that the iterator only yields submessages with the right parameter
pub(crate) fn iter_messages_of_parameter<'a, R: Read>(
    grib: &'a Grib2<R>,
    discipline: u8,
    category: u8,
    parameter: u8,
) -> impl Iterator<Item = (MessageIndex, SubMessage<'a, R>)> + 'a {
    grib.iter().filter(move |(index, message)| {
        let d = message.indicator().discipline;

        let prod_def = message.prod_def();

        let Some(c) = prod_def.parameter_category() else {
            warn!(
                "Unsupported product definition template number: {}, skipping message {:?}",
                prod_def.prod_tmpl_num(),
                index
            );
            return false;
        };

        let Some(p) = prod_def.parameter_number() else {
            warn!(
                "Unsupported product definition template number: {}, skipping message {:?}",
                prod_def.prod_tmpl_num(),
                index
            );
            return false;
        };

        // If all three are matched, let the iterator return this message
        d == discipline && c == category && p == parameter
    })
}

pub(crate) fn get_grid_and_values(
    byte_string: &[u8],
    message_index: (usize, usize),
) -> Result<(GridDefinitionTemplateValues, Vec<f32>), GribViewerError> {
    // Parse the GRIB2 message.
    let grib2 = grib::from_bytes(byte_string)?;

    // Find the target submessage.
    let (_index, submessage) = grib2
        .iter()
        .find(|(index, _)| *index == message_index)
        .ok_or_else(|| {
            GribViewerError::MessageNotFound(format!(
                "Index {:?} not found in Grib file",
                message_index
            ))
        })?;

    let grid_def = submessage.grid_def();
    let grid = GridDefinitionTemplateValues::try_from(grid_def)?;

    // Prepare a decoder.
    let decoder = grib::Grib2SubmessageDecoder::from(submessage)?;

    // Actually dispatch a decoding process and get an iterator of decoded values.
    // There are various methods available for compressing GRIB2 data, but some are
    // not yet supported by this library and may return errors.
    let values_iterator = decoder.dispatch()?;

    // extract values from iterator
    let values = values_iterator.collect();

    Ok((grid, values))
}

pub(crate) fn get_lat_lon_and_values(
    byte_string: &[u8],
    message_index: (usize, usize),
) -> Result<(Vec<f32>, Vec<f32>, Vec<f32>), GribViewerError> {
    // Parse the GRIB2 message.
    let grib2 = grib::from_bytes(byte_string)?;

    // Find the target submessage.
    let (_index, submessage) = grib2
        .iter()
        .find(|(index, _)| *index == message_index)
        .ok_or_else(|| {
            GribViewerError::MessageNotFound(format!(
                "Index {:?} not found in Grib file",
                message_index
            ))
        })?;

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
    let values = values_iterator.collect();

    Ok((lats, lons, values))
}

pub(crate) fn grib_parameter_from_key(key: &str) -> Result<(u8, u8, u8), GribViewerError> {
    let parts: Vec<&str> = key.split('_').collect();
    if parts.len() != 4 || parts[0] != "grib2" {
        return Err(GribViewerError::InvalidKey(
            "invalid key format, expected 'grib2_<discipline>_<category>_<parameter>'".to_string(),
        ));
    }
    let discipline: u8 = parts[1]
        .parse()
        .map_err(|e| GribViewerError::InvalidKey(format!("Failed to parse discipline: {}", e)))?;
    let category: u8 = parts[2]
        .parse()
        .map_err(|e| GribViewerError::InvalidKey(format!("Failed to parse category: {}", e)))?;
    let parameter: u8 = parts[3]
        .parse()
        .map_err(|e| GribViewerError::InvalidKey(format!("Failed to parse parameter: {}", e)))?;
    Ok((discipline, category, parameter))
}

pub(crate) fn find_grib_index(
    bytes: &[u8],
    discipline: u8,
    category: u8,
    parameter: u8,
    time: i64,
) -> Result<(usize, usize), GribViewerError> {
    let grib2 = grib::from_bytes(bytes)?;
    for ((index, subindex), message) in grib2.iter() {
        let d = message.indicator().discipline;
        let prod_def = message.prod_def();
        let Some(c) = prod_def.parameter_category() else {
            warn!(
                "Unsupported product definition template number: {}, skipping message",
                prod_def.prod_tmpl_num()
            );
            continue;
        };
        let p = prod_def
            .parameter_number()
            .expect("parameter_category() should have failed");

        let temporal_info = grib::TemporalInfo::from(&message.temporal_raw_info());
        let Some(t) = temporal_info.forecast_time_target else {
            warn!(
                "Message with invalid forecast time, skipping message {:?}",
                index
            );
            continue;
        };

        if d == discipline && c == category && p == parameter && t.timestamp() == time {
            return Ok((index, subindex));
        }
    }
    Err(GribViewerError::MessageNotFound(format!(
        "No message with: disc: {}, cat: {}, param: {}, time: {}",
        discipline, category, parameter, time
    )))
}

/// For regular grids: get the occuring lats and longs from lowest to highest.
///
/// Usually values will be between 0 and 360 degrees, except when they cross 0 longitude, then longitudes can be below
/// 0, to ensure that the longitudes do not jump from 360 to 0.
pub(crate) fn get_lat_lon_1d_without_jump(
    grid: &GridDefinitionTemplateValues,
) -> Result<(Vec<f32>, Vec<f32>), GribViewerError> {
    let (ni, nj) = grid.grid_shape();
    let mut lat = vec![0_f32; nj as usize];
    let mut lon = vec![0_f32; ni as usize];
    match grid {
        GridDefinitionTemplateValues::Template0(grid) => {
            // TODO: extracting the 1d lats and lons is convoluted, but there doesn't seem to be an easy way to do it.
            // The RegularGridIterator has the two arrays we are looking for as fields, but they are private. Perhaps
            // open an issue on grib-rs?
            let latlons = grid.latlons()?;
            let (lat_2d, lon_2d): (Vec<f32>, Vec<f32>) = latlons.unzip();
            for (idx, (i, j)) in grid.ij()?.enumerate() {
                if i == 0 {
                    lat[j] = lat_2d[idx];
                }
                if j == 0 {
                    lon[i] = lon_2d[idx];
                }
            }
        }
        GridDefinitionTemplateValues::Template20(_) => {
            return Err(GribViewerError::Other(
                "1d lat/lon not implemented for Polar Stereographic grid".into(),
            ));
        }
        GridDefinitionTemplateValues::Template30(_) => {
            // Lambert grid logic here
            return Err(GribViewerError::Other(
                "1d lat/lon not implemented for Lambert grid".into(),
            ));
        }
        GridDefinitionTemplateValues::Template40(grid) => {
            let latlons = grid.latlons()?;
            let (lat_2d, lon_2d): (Vec<f32>, Vec<f32>) = latlons.unzip();
            for (i, j) in grid.ij()? {
                if i == 0 {
                    lat[j] = lat_2d[j];
                }
                if j == 0 {
                    lon[i] = lon_2d[i];
                }
            }
        }
    }
    if lon[0] > lon[ni - 1] {
        for i in 0..nj {
            if lon[i] > lon[ni - 1] {
                lon[i] -= 360.0;
            } else {
                break;
            }
        }
    }
    return Ok((lat, lon));
}
