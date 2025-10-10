use std::f64::consts::PI;
use num::{Float, NumCast, FromPrimitive};

static EARTH_RADIUS: f64 = 6378137.0;
static MAX_LATITUDE: f64 = 85.0511287798;
static FALSE_EASTING: f64 = 0.0;
static FALSE_NORTHING: f64 = 0.0;

/// epsg 3857 projection according to: Geomatics Guidance Note number 7, part 2, section 3.2.1.2
pub fn epsg_3857_projection<T>(lat: T, lon: T) -> (T, T)
where
    T: Float + NumCast + FromPrimitive,
{
    // Clamp the latitude
    let clamped_lat = lat.clamp(
        T::from_f64(-MAX_LATITUDE).unwrap(),
        T::from_f64(MAX_LATITUDE).unwrap(),
    );

    // Calculate x and y
    let x = T::from_f64(FALSE_EASTING).unwrap()
        + T::from_f64(EARTH_RADIUS).unwrap() * lon.to_radians();
    let y = T::from_f64(FALSE_NORTHING).unwrap()
        + T::from_f64(EARTH_RADIUS).unwrap()
        * (T::from_f64(PI / 4.0).unwrap() + clamped_lat.to_radians() / T::from_f64(2.0).unwrap())
            .tan()
            .ln();

    (x, y)
}

/// inverse epsg 3857 projection according to: Geomatics Guidance Note number 7, part 2, section 3.2.1.2
pub fn inverse_epsg_3857_projection<T>(x: T, y: T) -> (T, T)
where
    T: Float + NumCast + FromPrimitive,
{
    let d = (T::from_f64(FALSE_NORTHING).unwrap() - y) / T::from_f64(EARTH_RADIUS).unwrap();
    let lat = (T::from_f64(PI/2.0).unwrap() - T::from_f64(2.0).unwrap() * d.exp().atan()).to_degrees();
    let lon = ((x - T::from_f64(FALSE_EASTING).unwrap()) / T::from_f64(EARTH_RADIUS).unwrap()).to_degrees();
    (lat, lon)
}

/// Distance (meters) between two points (latitude and longitude in degrees)
pub fn haversine_distance(lat1: f32, lon1: f32, lat2: f32, lon2: f32) -> f32 {
    let a = ((lat2 - lat1) / 2.0).to_radians().sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * ((lon2 - lon1) / 2.0).to_radians().sin().powi(2);
    2.0 * f32::from_f64(EARTH_RADIUS).unwrap() * a.sqrt().asin()
}

#[cfg(test)]
mod tests {
    use crate::projection::{epsg_3857_projection, inverse_epsg_3857_projection};

    static TOL: f64 = 0.01;
    #[test]
    fn test_epsg_3857_projection() {
        // example from page 45 of Geomatics Guidance Note number 7, part 2
        let lon = f64::from(-1.751147016).to_degrees();
        let lat = f64::from(0.425542460).to_degrees();
        let (x, y) = epsg_3857_projection(lat, lon);
        assert!(x - -11169055.58 < TOL);
        assert!(y - 2800000.00 < TOL);
    }

    #[test]
    fn test_inverse_epsg_3857_projection() {
        // example from page 45 of Geomatics Guidance Note number 7, part 2
        let x =  -11169055.58;
        let y = 2810000.00;
        let (lat, lon) = inverse_epsg_3857_projection(x, y);
        assert!(lat - f64::from(0.426970023).to_degrees() < TOL);
        assert!(lon - f64::from(-1.751147016).to_degrees() < TOL);
    }
}
