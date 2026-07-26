use crate::Point;

pub(crate) const MAX_RECURSION: u32 = 16;

pub(crate) fn flatten_cubic(
    output: &mut Vec<Point>,
    p0: Point,
    p1: Point,
    p2: Point,
    p3: Point,
    max_flatness_squared: f32,
    depth: u32,
) {
    let d0 = p1 - p0;
    let d1 = p2 - p1;
    let d2 = p3 - p2;
    let direct = p3 - p0;
    let long_length = d0.length() + d1.length() + d2.length();
    let short_length = direct.length();
    let flatness_squared = long_length * long_length - short_length * short_length;

    if flatness_squared > max_flatness_squared && depth < MAX_RECURSION {
        let p01 = midpoint(p0, p1);
        let p12 = midpoint(p1, p2);
        let p23 = midpoint(p2, p3);
        let a = midpoint(p01, p12);
        let b = midpoint(p12, p23);
        let middle = midpoint(a, b);

        flatten_cubic(
            output,
            p0,
            p01,
            a,
            middle,
            max_flatness_squared,
            depth + 1,
        );
        flatten_cubic(
            output,
            middle,
            b,
            p23,
            p3,
            max_flatness_squared,
            depth + 1,
        );
    } else {
        output.push(p3);
    }
}

pub(crate) fn flatten_quadratic(
    output: &mut Vec<Point>,
    p0: Point,
    p1: Point,
    p2: Point,
    max_flatness_squared: f32,
    depth: u32,
) {
    let middle = Point::new(
        (p0.x + 2.0 * p1.x + p2.x) * 0.25,
        (p0.y + 2.0 * p1.y + p2.y) * 0.25,
    );
    let chord_middle = midpoint(p0, p2);
    let delta = chord_middle - middle;
    let flatness_squared = delta.dot(delta);

    if flatness_squared > max_flatness_squared && depth < MAX_RECURSION {
        let p01 = midpoint(p0, p1);
        let p12 = midpoint(p1, p2);
        flatten_quadratic(
            output,
            p0,
            p01,
            middle,
            max_flatness_squared,
            depth + 1,
        );
        flatten_quadratic(
            output,
            middle,
            p12,
            p2,
            max_flatness_squared,
            depth + 1,
        );
    } else {
        output.push(p2);
    }
}

fn midpoint(a: Point, b: Point) -> Point {
    Point::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5)
}
