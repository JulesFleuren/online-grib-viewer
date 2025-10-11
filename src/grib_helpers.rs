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
