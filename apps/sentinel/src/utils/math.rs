pub fn euclidean_distance(point1: (i32, i32), point2: (i32, i32)) -> i32 {
    ((point2.0 - point1.0).pow(2) + (point2.1 - point1.1).pow(2)).isqrt() as i32
}
