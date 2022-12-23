use anyhow::Result;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct Point([isize; 3]);

impl std::str::FromStr for Point {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts = s.split(',')
                     .map(|x| x.parse()
                               .map_err(|e| anyhow::anyhow!("Invalid coord: {}", e)))
                     .collect::<Result<Vec<_>>>()?;
        Ok(Self(parts.try_into().map_err(|_| anyhow::anyhow!("Wrong number of coordinates"))?))
    }
}

fn solve1(input: &Input) -> Result<usize> {
    let data = input.iter().cloned().collect::<std::collections::HashSet<_>>();
    let sides = input.iter().cloned()
               .map(|Point([x, y, z])| {
                   !data.contains(&Point([x - 1, y, z])) as usize +
                   !data.contains(&Point([x + 1, y, z])) as usize +
                   !data.contains(&Point([x, y - 1, z])) as usize +
                   !data.contains(&Point([x, y + 1, z])) as usize +
                   !data.contains(&Point([x, y, z - 1])) as usize +
                   !data.contains(&Point([x, y, z + 1])) as usize
               })
               .sum();

    Ok(sides)
}

fn solve2(input: &Input) -> Result<usize> {
    let bound = input.iter().cloned().flat_map(|p| p.0).max().unwrap() + 1;

    // volume in the cube where all coords are between -1 and bound (inclusive)
    //
    // simple flood-fill
    let data = input.iter().cloned().collect::<std::collections::HashSet<_>>();
    let mut points = vec![[-1, -1, -1]]; // stack of points outside the boundary
    let mut air = std::collections::HashSet::new();

    let range = -1..=bound;
    while let Some(p@[x,y,z]) = points.pop() {
        air.insert(p);

        for delta in [-1isize, 1isize] {
            if !data.contains(&Point([x+delta, y, z])) &&
                range.contains(&(x+delta)) &&
                !air.contains(&[x+delta, y, z])
            {
                points.push([x+delta, y, z]);
            }
            if !data.contains(&Point([x, y+delta, z])) &&
                range.contains(&(y+delta)) &&
                !air.contains(&[x, y+delta, z])
            {
                points.push([x, y+delta, z]);
            }
            if !data.contains(&Point([x, y, z+delta])) &&
                range.contains(&(z+delta)) &&
                !air.contains(&[x, y, z+delta])
            {
                points.push([x, y, z+delta]);
            }
        }
    }

    Ok(input.iter().cloned()
            .map(|Point([x, y, z])| {
                air.contains(&[x - 1, y, z]) as usize +
                air.contains(&[x + 1, y, z]) as usize +
                air.contains(&[x, y - 1, z]) as usize +
                air.contains(&[x, y + 1, z]) as usize +
                air.contains(&[x, y, z - 1]) as usize +
                air.contains(&[x, y, z + 1]) as usize
            }).sum())
}

problem!(crate::util::load_lines => Vec<Point> => (solve1, solve2));
