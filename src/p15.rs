use anyhow::Result;

use std::collections::BinaryHeap;

fn load_input(input: &mut dyn std::io::BufRead) -> Result<Input> {
    crate::util::read_lines_regex(
        input,
        r#"^Sensor at x=(-?\d+), y=(-?\d+): closest beacon is at x=(-?\d+), y=(-?\d+)$"#,
        |caps| {
            let px = caps.get(1).unwrap().as_str().parse::<isize>()?;
            let py = caps.get(2).unwrap().as_str().parse::<isize>()?;
            let bx = caps.get(3).unwrap().as_str().parse::<isize>()?;
            let by = caps.get(4).unwrap().as_str().parse::<isize>()?;

            Ok(Reading { position: (px, py), beacon: (bx, by) })
        }
    )
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
/// Span of cells `[start, end)`
struct Span {
    start: isize,
    end: isize,
}

#[derive(Debug)]
struct Reading {
    /// Sensor position
    position: (isize, isize),

    /// Closest beacon to this position
    beacon: (isize, isize),
}

impl Reading {
    /// Return the radius of this reading
    ///
    /// This uses the Manhattan distance, per the problem description.
    fn radius(&self) -> isize {
        (self.position.0 - self.beacon.0).abs() + (self.position.1 - self.beacon.1).abs()
    }

    /// Compute the span of X-coordinates covered by this beacon at a given Y coordinate
    fn span_at(&self, y: isize) -> Span {
        let rad = self.radius();
        let dy = (self.position.1 - y).abs();

        // span width decreases by one on each side for each one extra Y-delta
        let width = ((2*rad + 1) - 2*dy).max(0); 
        let half = (width - 1) / 2;
        let start = self.position.0 - half;
        let end = if width == 0 {
            self.position.0
        } else {
            self.position.0 + half + 1
        };

        Span { start, end }
    }
}

/// Run a function taking a list of relevant beacons over each row in a range
///
/// For each row in `[min, max)`, call the passed function with the Y coordinate and a set of
/// relevant beacon indices.
fn relevant_beacons<F>(input: &Input, min: isize, max: isize, mut func: F)
where F: FnMut(isize, &std::collections::HashSet<usize>) {
    struct Event {
        /// Whether the reading becomes irrelevant (`false`) or relevant (`true`)
        relevant: bool,
        y: isize,

        /// Reading index
        reading: usize,
    }

    impl std::cmp::PartialEq for Event {
        fn eq(&self, other: &Self) -> bool {
            self.y == other.y
        }
    }

    impl std::cmp::Eq for Event {}

    impl std::cmp::Ord for Event {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.y.cmp(&other.y).reverse() // we want a min-heap, not a max-heap
        }
    }

    impl std::cmp::PartialOrd for Event {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    // Set up event tracking with events when each beacon becomes relevant
    let mut events = BinaryHeap::with_capacity(input.len());
    events.extend(input.iter().enumerate().map(|(idx, b)| Event {
        relevant: true,
        y: b.position.1 - b.radius(),
        reading: idx,
    }));

    // Traverse event sequence to find the set of relevant beacons in the target row
    let mut relevant = std::collections::HashSet::new();

    let mut next_row = min; // next row to run function over

    while let Some(event) = events.pop() {
        while event.y > next_row && next_row < max {
            // evaluate function over the current row
            (func)(next_row, &relevant);

            next_row += 1;
        }
        if next_row >= max {
            // no more outputs needed - we're done
            break;
        }

        if event.relevant {
            // add relevant reading, and register an event for when it becomes irrelevant
            relevant.insert(event.reading);
            let reading = &input[event.reading];
            events.push(Event {
                relevant: false,
                y: reading.position.1 + reading.radius(),
                reading: event.reading,
            });
        } else {
            // remove relevant reading
            relevant.remove(&event.reading);
        }
    }
}

/// Compute beacon spans for each row in a range
///
/// Compute the spans covered by observed beacons for each row in `y=[min, max)`. Returns a vector
/// of `max-min` vectors, each one containing a list of spans. Span lists are sorted by their
/// starting X coordinate.
fn beacon_spans(input: &Input, min: isize, max: isize) -> Vec<Vec<Span>> {
    let mut output = Vec::with_capacity((max - min).max(0) as usize);
    let mut covered = Vec::new();
    relevant_beacons(input, min, max, |row, relevant| {
        // Compute the covered span for each relevant beacon
        covered.clear();
        covered.extend(relevant.iter()
                               .map(|idx| input[*idx].span_at(row))
                               .filter(|span| span.end > span.start));

        // Merge overlapping spans
        covered.sort_unstable_by_key(|span| span.start);

        let mut final_spans = Vec::new();
        let mut cur_span: Option<Span> = None;
        for span in covered.drain(..) {
            if let Some(s) = cur_span.as_mut() {
                if s.end >= span.start {
                    // merge spans
                    s.end = span.end.max(s.end);
                } else {
                    // spans are separate
                    final_spans.extend(cur_span.take());
                    cur_span = Some(span);
                }
            } else {
                cur_span = Some(span);
            }
        }
        final_spans.extend(cur_span);

        output.push(final_spans);
    });

    output
}

/// Find number of spaces which cannot contain a beacon within row R
///
/// This operates by sweeping a line downwards from y=0 towards the target row, maintaining a list
/// of relevant beacons at each step. When the target row is reached, each relevant beacon is
/// evaluated to give a span of X-coordinates that it covers. Overlapping spans are then merged, and
/// the final result is computed as the sum of all span widths.
fn solve1(input: &Input) -> Result<usize> {
    const TARGET_ROW: isize = 2_000_000;
    //const TARGET_ROW: isize = 10;

    anyhow::ensure!(!input.is_empty());

    let spans = beacon_spans(input, TARGET_ROW, TARGET_ROW+1).swap_remove(0);

    // Remove any beacons within this row
    let mut beacons = input.iter()
                     .map(|r| r.beacon)
                     .filter(|b| b.1 == TARGET_ROW)
                     .collect::<Vec<_>>();
    beacons.sort();
    beacons.dedup();

    let beacons = beacons.len();

    Ok(spans.iter().map(|span| (span.end - span.start) as usize).sum::<usize>() - beacons)
}

fn solve2(input: &Input) -> Result<usize> {
    const BOUND: isize = 4_000_000;
    //const BOUND: isize = 20;

    anyhow::ensure!(!input.is_empty());

    let mut out_coords = Vec::new();
    for (row, spans) in beacon_spans(input, 0, BOUND+1).iter().enumerate() {
        if spans.len() == 1 {
            // does the row's single span cover all cells?
            if spans[0].start <= 0 && spans[0].end > BOUND {
                continue;
            }
        }

        // figure out which cell is empty
        let mut x = 0; // next candidate cell (i.e. cell that might be free)
        for span in spans {
            if span.start == x+1 {
                out_coords.push((x as usize, row));
                x = span.end;
            } else if span.start <= x {
                x = span.end;
            } else {
                anyhow::bail!("Too many possible solutions - problem is underconstrained");
            }
        }

        if x <= BOUND {
            for x in x..=BOUND {
                out_coords.push((x as usize, row));
            }
        }
    }

    anyhow::ensure!(!out_coords.is_empty(), "No solutions found");
    anyhow::ensure!(out_coords.len() == 1, "Multiple solutions - unable to select single point");
    Ok(out_coords[0].0 * 4_000_000 + out_coords[0].1)
}

problem!(load_input => Vec<Reading> => (solve1, solve2));
