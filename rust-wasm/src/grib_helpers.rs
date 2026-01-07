use grib::codetables::{CodeTable4_5, Lookup};
use grib::{
    FixedSurface, Grib2, Grib2Read, GridDefinitionTemplateValues, MessageIndex, SubMessage,
};

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

pub(crate) fn iter_messages_of_parameter_and_surface<'a, R: Read>(
    grib: &'a Grib2<R>,
    discipline: u8,
    category: u8,
    parameter: u8,
    surface1: FixedSurface,
    surface2: FixedSurface,
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
        if let Some((s1, s2)) = prod_def.fixed_surfaces() {
            return d == discipline
                && c == category
                && p == parameter
                && s1 == surface1
                && s2 == surface2;
        } else {
            warn!(
                "Unsupported product definition template number: {}, skipping message {:?}",
                prod_def.prod_tmpl_num(),
                index
            );
            return false;
        }
    })
}

pub(crate) fn get_message<'a, R: Read>(
    grib: &'a Grib2<R>,
    discipline: u8,
    category: u8,
    parameter: u8,
    surface1: &FixedSurface,
    surface2: &FixedSurface,
    time: i64,
) -> Result<SubMessage<'a, R>, GribViewerError> {
    for (index, message) in grib.iter() {
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

        let Some((s1, s2)) = prod_def.fixed_surfaces() else {
            warn!(
                "Unsupported product definition template number: {}, skipping message {:?}",
                prod_def.prod_tmpl_num(),
                index
            );
            continue;
        };
        if d == discipline
            && c == category
            && p == parameter
            && t.timestamp() == time
            && s1 == *surface1
            && s2 == *surface2
        {
            return Ok(message);
        }
    }
    Err(GribViewerError::MessageNotFound(format!(
        "No message with: disc: {}, cat: {}, param: {}, time: {}",
        discipline, category, parameter, time
    )))
}

pub(crate) fn get_grid_and_values<R: Grib2Read>(
    submessage: SubMessage<'_, R>,
) -> Result<(GridDefinitionTemplateValues, Vec<f32>), GribViewerError> {
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

pub(crate) fn get_lat_lon_and_values<R: Grib2Read>(
    submessage: SubMessage<'_, R>,
) -> Result<(Vec<f32>, Vec<f32>, Vec<f32>), GribViewerError> {
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

pub(crate) fn fixed_surfaces_from_key(
    key: &str,
) -> Result<(FixedSurface, FixedSurface), GribViewerError> {
    let parts: Vec<&str> = key.split('_').collect();
    if parts.len() != 7 || parts[0] != "surface" {
        return Err(GribViewerError::InvalidKey(
            "invalid key format, expected 'surface_<t1>_<f1>_<v1>_<t2>_<f2>_<v2>'".to_string(),
        ));
    }
    let surface_type1: u8 = parts[1]
        .parse()
        .map_err(|e| GribViewerError::InvalidKey(format!("Failed to parse surface_type: {}", e)))?;
    let scale_factor1: i8 = parts[2]
        .parse()
        .map_err(|e| GribViewerError::InvalidKey(format!("Failed to parse scale_factor: {}", e)))?;
    let scaled_value1: i32 = parts[3]
        .parse()
        .map_err(|e| GribViewerError::InvalidKey(format!("Failed to parse scaled_value: {}", e)))?;
    let surface_type2: u8 = parts[4]
        .parse()
        .map_err(|e| GribViewerError::InvalidKey(format!("Failed to parse surface_type: {}", e)))?;
    let scale_factor2: i8 = parts[5]
        .parse()
        .map_err(|e| GribViewerError::InvalidKey(format!("Failed to parse scale_factor: {}", e)))?;
    let scaled_value2: i32 = parts[6]
        .parse()
        .map_err(|e| GribViewerError::InvalidKey(format!("Failed to parse scaled_value: {}", e)))?;

    Ok((
        FixedSurface {
            surface_type: surface_type1,
            scale_factor: scale_factor1,
            scaled_value: scaled_value1,
        },
        FixedSurface {
            surface_type: surface_type2,
            scale_factor: scale_factor2,
            scaled_value: scaled_value2,
        },
    ))
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

pub(crate) fn format_surfaces(surface1: &FixedSurface, surface2: &FixedSurface) -> String {
    let type1 = CodeTable4_5
        .lookup(usize::from(surface1.surface_type))
        .to_string();
    let value1 = surface1.value();
    let unit1 = surface1.unit().unwrap_or("");

    let surface1_string = format!("{}: {}{}", type1, value1, unit1);
    if surface2.surface_type == 255 {
        // surface 2 is missing: only format surface 1 to string
        return surface1_string;
    } else {
        let type2 = CodeTable4_5
            .lookup(usize::from(surface2.surface_type))
            .to_string();
        let value2 = surface2.value();
        let unit2 = surface2.unit().unwrap_or("");
        return format!("{} - {}: {}{}", surface1_string, type2, value2, unit2);
    }
}
