use num::{Float, FromPrimitive, NumCast};
use std::f64::consts::PI;

static EARTH_RADIUS: f64 = 6378137.0;
static MAX_LATITUDE: f64 = 85.0511287798; // with this bound, the map is twice as wide as it is high

pub(crate) struct Epsg3857Projection {
    earth_radius: f64,
    max_latitude: f64,
    false_northing: f64,
    false_easting: f64,
}

impl Default for Epsg3857Projection {
    /// Projection with earth radius, that leads to a projection that is twice as wid as it is high. Projected
    /// values will be positive if lon is between -360 and 360
    fn default() -> Self {
        Epsg3857Projection {
            earth_radius: EARTH_RADIUS,
            max_latitude: MAX_LATITUDE,
            false_northing: EARTH_RADIUS * PI,
            false_easting: EARTH_RADIUS * 2.0 * PI,
        }
    }
}

impl Epsg3857Projection {
    /// Projection with unit radius. Projected values will be positive if lon is between -360 and 360.
    pub fn unit_projection() -> Self {
        Epsg3857Projection {
            earth_radius: 1.0,
            max_latitude: MAX_LATITUDE,
            false_northing: PI,
            false_easting: 2.0 * PI,
        }
    }

    pub fn from_radius(radius: f64) -> Self {
        Epsg3857Projection {
            earth_radius: radius,
            max_latitude: MAX_LATITUDE,
            false_northing: radius * PI,
            false_easting: radius * 2.0 * PI,
        }
    }

    /// epsg 3857 projection according to: Geomatics Guidance Note number 7, part 2, section 3.2.1.2
    pub fn project<T>(&self, lat: T, lon: T) -> (T, T)
    where
        T: Float + NumCast + FromPrimitive,
    {
        // Clamp the latitude
        let clamped_lat = lat.clamp(
            T::from_f64(-self.max_latitude).unwrap(),
            T::from_f64(self.max_latitude).unwrap(),
        );

        // Calculate x and y
        let x = T::from_f64(self.false_easting).unwrap()
            + T::from_f64(self.earth_radius).unwrap() * lon.to_radians();
        let y = T::from_f64(self.false_northing).unwrap()
            + T::from_f64(self.earth_radius).unwrap()
                * (T::from_f64(PI / 4.0).unwrap()
                    + clamped_lat.to_radians() / T::from_f64(2.0).unwrap())
                .tan()
                .ln();

        (x, y)
    }

    /// inverse epsg 3857 projection according to: Geomatics Guidance Note number 7, part 2, section 3.2.1.2
    pub fn unproject<T>(&self, x: T, y: T) -> (T, T)
    where
        T: Float + NumCast + FromPrimitive,
    {
        let d = (T::from_f64(self.false_northing).unwrap() - y)
            / T::from_f64(self.earth_radius).unwrap();
        let lat = (T::from_f64(PI / 2.0).unwrap() - T::from_f64(2.0).unwrap() * d.exp().atan())
            .to_degrees();
        let lon = ((x - T::from_f64(self.false_easting).unwrap())
            / T::from_f64(self.earth_radius).unwrap())
        .to_degrees();
        (lat, lon)
    }
}

/// Distance (meters) between two points (latitude and longitude in degrees)
pub fn haversine_distance(lat1: f32, lon1: f32, lat2: f32, lon2: f32) -> f32 {
    let a = ((lat2 - lat1) / 2.0).to_radians().sin().powi(2)
        + lat1.to_radians().cos()
            * lat2.to_radians().cos()
            * ((lon2 - lon1) / 2.0).to_radians().sin().powi(2);
    2.0 * f32::from_f64(EARTH_RADIUS).unwrap() * a.sqrt().asin()
}

#[cfg(test)]
mod tests {
    use crate::projection::{EARTH_RADIUS, Epsg3857Projection, MAX_LATITUDE};

    static TOL: f64 = 0.01;
    #[test]
    fn test_epsg_3857_projection() {
        // example from page 45 of Geomatics Guidance Note number 7, part 2
        let projection = Epsg3857Projection {
            earth_radius: EARTH_RADIUS,
            max_latitude: MAX_LATITUDE,
            false_easting: 0.0,
            false_northing: 0.0,
        };
        let lon = f64::from(-1.751147016).to_degrees();
        let lat = f64::from(0.425542460).to_degrees();
        let (x, y) = projection.project(lat, lon);
        assert!(x - -11169055.58 < TOL);
        assert!(y - 2800000.00 < TOL);
    }

    #[test]
    fn test_inverse_epsg_3857_projection() {
        // example from page 45 of Geomatics Guidance Note number 7, part 2
        let projection = Epsg3857Projection {
            earth_radius: EARTH_RADIUS,
            max_latitude: MAX_LATITUDE,
            false_easting: 0.0,
            false_northing: 0.0,
        };
        let x = -11169055.58;
        let y = 2810000.00;
        let (lat, lon) = projection.unproject(x, y);
        assert!(lat - f64::from(0.426970023).to_degrees() < TOL);
        assert!(lon - f64::from(-1.751147016).to_degrees() < TOL);
    }

    #[test]
    fn test_default_epsg_3857_projection_always_positive() {
        let projection = Epsg3857Projection::default();
        let lats = vec![-90.0, 0.0, 90.0];
        let lons = vec![-360.0, 0.0, 360.0];
        for lat in &lats {
            for lon in &lons {
                let (x, y) = projection.project(*lat, *lon);
                assert!(x >= 0.0, "x = {}, lat = {}, lon = {}", x, lat, lon);
                assert!(y >= 0.0, "y = {}, lat = {}, lon = {}", y, lat, lon);
            }
        }
    }

    #[test]
    fn test_unit_epsg_3857_projection_always_positive() {
        let projection = Epsg3857Projection::unit_projection();
        let lats = vec![-90.0, 0.0, 90.0];
        let lons = vec![-360.0, 0.0, 360.0];
        for lat in &lats {
            for lon in &lons {
                let (x, y) = projection.project(*lat, *lon);
                assert!(x >= 0.0, "x = {}, lat = {}, lon = {}", x, lat, lon);
                assert!(y >= 0.0, "y = {}, lat = {}, lon = {}", y, lat, lon);
            }
        }
    }

    #[test]
    fn test_from_radius_epsg_3857_projection_always_positive() {
        let projection = Epsg3857Projection::from_radius(250_f64.to_radians());
        let lats = vec![-90.0, 0.0, 90.0];
        let lons = vec![-360.0, 0.0, 360.0];
        for lat in &lats {
            for lon in &lons {
                let (x, y) = projection.project(*lat, *lon);
                assert!(x >= 0.0, "x = {}, lat = {}, lon = {}", x, lat, lon);
                assert!(y >= 0.0, "y = {}, lat = {}, lon = {}", y, lat, lon);
            }
        }
    }

    #[test]
    fn test_project_unproject() {
        // some arbitrary values
        let projection = Epsg3857Projection {
            earth_radius: 100.0,
            max_latitude: MAX_LATITUDE,
            false_easting: 700.0,
            false_northing: 300000.0,
        };
        let lat: f64 = 12.3456;
        let lon: f64 = 234.567;
        let (x, y) = projection.project(lat, lon);
        let (lat2, lon2) = projection.unproject(x, y);
        assert!((lat - lat2).abs() < TOL);
        assert!((lon - lon2).abs() < TOL);
    }
}
