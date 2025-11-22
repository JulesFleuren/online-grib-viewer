use crate::projection::haversine_distance;

pub(crate) fn norm(first_component: &Vec<f32>, second_component: &Vec<f32>) -> Vec<f32> {
    // TODO: check if lengths match
    let mut result = Vec::with_capacity(first_component.len());
    for idx in 0..first_component.len() {
        result.push(f32::sqrt(
            first_component[idx].powi(2) + second_component[idx].powi(2),
        ));
    }
    result
}

pub(crate) fn find_closest_point_in_grid(
    lat: &Vec<f32>,
    lon: &Vec<f32>,
    query_lat: f32,
    query_lon: f32,
) -> usize {
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
