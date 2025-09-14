use std::f64::consts::PI;

use log::debug;

static EARTH_RADIUS: f64 = 6378137.0;
static MAX_LATITUDE: f64 = 85.0511287798;
static FALSE_EASTING: f64 = 0.0;
static FALSE_NORTHING: f64 = 0.0;

pub fn epsg_3857_projection(lat: f64, lon: f64) -> (f64, f64) {
    // epsg 3857 projection according to: Geomatics Guidance Note number 7, part 2, section 3.2.1.2
    let clamped_lat = lat.clamp(-MAX_LATITUDE, MAX_LATITUDE);
    let x = FALSE_EASTING + EARTH_RADIUS * lon.to_radians();
    let y = FALSE_NORTHING + EARTH_RADIUS * (PI/4.0 + clamped_lat.to_radians()/2.0).tan().ln();
    if x.is_nan() || y.is_nan() || x.is_infinite() || y.is_infinite() {
        debug!("x: {}, y: {}, lat: {}, lon: {}", x, y, lat, lon);
        debug!("{} {} {} {} {} {}",
        clamped_lat,
        clamped_lat.to_radians()/2.0,
        (PI/4.0 + clamped_lat.to_radians()/2.0),
        (PI/4.0 + clamped_lat.to_radians()/2.0).tan(),
        (PI/4.0 + clamped_lat.to_radians()/2.0).tan().ln(),
        FALSE_NORTHING + EARTH_RADIUS * (PI/4.0 + clamped_lat.to_radians()/2.0).tan().ln()
    );
    }
    (x, y)
}

pub fn inverse_epsg_3857_projection(x: f64, y: f64) -> (f64, f64) {
    let d = (FALSE_NORTHING - y) / EARTH_RADIUS;
    let lat = (PI/2.0 - 2.0 * d.exp().atan()).to_degrees();
    let lon = ((x - FALSE_EASTING) / EARTH_RADIUS).to_degrees();
    if lat.is_nan() || lon.is_nan() {
        debug!("lat: {}, lon: {}, x: {}, y: {}", lat, lon, x, y)
    }
    (lat, lon)
}
