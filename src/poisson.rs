use crate::{Error, Point, Viewport};

const MAX_ATTEMPTS: usize = 30;
const MAX_GRID_CELLS: usize = 16_000_000;

pub(crate) fn generate(viewport: Viewport, radius: f32, seed: u32) -> Result<Vec<Point>, Error> {
    let width = viewport.width();
    let height = viewport.height();
    let cell_size = radius / 2.0_f32.sqrt();
    let inverse_cell = 1.0 / cell_size;
    let columns = (width * inverse_cell).ceil().max(1.0) as usize;
    let rows = (height * inverse_cell).ceil().max(1.0) as usize;
    let cell_count = columns
        .checked_mul(rows)
        .filter(|count| *count <= MAX_GRID_CELLS)
        .ok_or(Error::SamplingGridTooLarge)?;

    let mut grid = vec![None; cell_count];
    let mut active = Vec::new();
    let mut samples = Vec::new();
    let mut state = seed;

    let first = Point::new(
        width * random_unit(next_seed(&mut state)),
        height * random_unit(next_seed(&mut state)),
    );
    let first_cell = grid_index(first, inverse_cell, columns, rows);
    grid[first_cell] = Some(0);
    active.push(0usize);
    samples.push(first);

    while !active.is_empty() && samples.len() < cell_count {
        let random_active = random_unit(next_seed(&mut state)) * active.len() as f32;
        let active_index = (random_active as usize).min(active.len() - 1);
        let sample_index = active[active_index];
        let center = samples[sample_index];
        let mut accepted = None;

        for _ in 0..MAX_ATTEMPTS {
            let candidate = sample_annulus(radius, center, &mut state);
            if candidate.x < 0.0
                || candidate.x >= width
                || candidate.y < 0.0
                || candidate.y >= height
            {
                continue;
            }

            if is_far_enough(
                candidate,
                radius,
                inverse_cell,
                columns,
                rows,
                &grid,
                &samples,
            ) {
                accepted = Some(candidate);
                break;
            }
        }

        if let Some(candidate) = accepted {
            let new_index = samples.len();
            let cell = grid_index(candidate, inverse_cell, columns, rows);
            grid[cell] = Some(new_index);
            active.push(new_index);
            samples.push(candidate);
        } else {
            active.swap_remove(active_index);
        }
    }

    for sample in &mut samples {
        sample.x += viewport.left;
        sample.y += viewport.top;
    }

    Ok(samples)
}

fn is_far_enough(
    point: Point,
    radius: f32,
    inverse_cell: f32,
    columns: usize,
    rows: usize,
    grid: &[Option<usize>],
    samples: &[Point],
) -> bool {
    let min_x = (((point.x - radius) * inverse_cell) as isize).clamp(0, columns as isize - 1);
    let min_y = (((point.y - radius) * inverse_cell) as isize).clamp(0, rows as isize - 1);
    let max_x = (((point.x + radius) * inverse_cell) as isize).clamp(0, columns as isize - 1);
    let max_y = (((point.y + radius) * inverse_cell) as isize).clamp(0, rows as isize - 1);
    let radius_squared = radius * radius;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let cell = y as usize * columns + x as usize;
            if let Some(sample_index) = grid[cell] {
                let delta = samples[sample_index] - point;
                if delta.dot(delta) < radius_squared {
                    return false;
                }
            }
        }
    }

    true
}

fn grid_index(
    point: Point,
    inverse_cell: f32,
    columns: usize,
    rows: usize,
) -> usize {
    let x = ((point.x * inverse_cell) as usize).min(columns - 1);
    let y = ((point.y * inverse_cell) as usize).min(rows - 1);
    y * columns + x
}

fn sample_annulus(radius: f32, center: Point, state: &mut u32) -> Point {
    loop {
        let x = 4.0 * random_unit(next_seed(state)) - 2.0;
        let y = 4.0 * random_unit(next_seed(state)) - 2.0;
        let candidate = Point::new(x, y);
        let squared = candidate.dot(candidate);
        if squared > 1.0 && squared <= 4.0 {
            return center + candidate * radius;
        }
    }
}

pub(crate) fn random_for_index(seed: u32, index: usize) -> f32 {
    random_unit(hash(seed.wrapping_add(index as u32)))
}

fn next_seed(state: &mut u32) -> u32 {
    let current = *state;
    *state = (*state).wrapping_add(1);
    current
}

fn random_unit(seed: u32) -> f32 {
    hash(seed) as f32 / u32::MAX as f32
}

fn hash(seed: u32) -> u32 {
    let mut value = (seed ^ 12_345_391).wrapping_mul(2_654_435_769);
    value ^= (value << 6) ^ (value >> 26);
    value = value.wrapping_mul(2_654_435_769);
    value = value.wrapping_add((value << 5) ^ (value >> 12));
    value
}
