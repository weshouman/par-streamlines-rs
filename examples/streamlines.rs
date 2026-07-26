use par_streamlines::{Config, Point, Tessellator, Viewport};

fn main() -> Result<(), par_streamlines::Error> {
    let tessellator = Tessellator::new(Config {
        thickness: 0.015,
        streamlines_seed_spacing: 0.18,
        streamlines_seed_viewport: Viewport::new(-1.0, -1.0, 1.0, 1.0),
        ..Config::default()
    })?;

    let mesh = tessellator.streamlines(0, 32, |point: &mut Point| {
        let x = point.x;
        point.x += -point.y * 0.025;
        point.y += x * 0.025;
    })?;

    println!("vertices: {}", mesh.vertex_count());
    println!("triangles: {}", mesh.triangle_count());
    Ok(())
}
