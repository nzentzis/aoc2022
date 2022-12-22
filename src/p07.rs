use anyhow::{anyhow, Result};

use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug)]
enum Node {
    /// File with the given size
    File(u64),

    /// Directory, containing named items
    Dir {
        total_size: u64,
        children: HashMap<String, Self>
    },
}

impl Node {
    fn update_sizes(&mut self) -> u64 {
        match self {
            Self::File(n) => *n,
            Self::Dir {total_size, children} => {
                *total_size = children.values_mut()
                             .map(|child| child.update_sizes())
                             .sum();
                *total_size
            },
        }
    }

    /// Run a function over all dirs
    fn on_dirs<F: FnMut(u64, &HashMap<String, Self>)>(&self, func: &mut F) {
        if let Self::Dir {total_size, children} = self {
            for child in children.values() {
                child.on_dirs(func);
            }
            (func)(*total_size, children);
        }
    }
}

fn load_input(input: &mut dyn std::io::BufRead) -> Result<Input> {
    let lines = crate::util::read_lines(input, |line| Ok(line.to_owned()))?;

    fn build_tree<'a, I: Iterator<Item=&'a str>>(
        lines: &mut I,
        root: &mut HashMap<String, Node>
    ) -> Result<()> {
        while let Some(line) = lines.next() {
            if line.starts_with("$ cd ") {
                // move up or down
                let name = line.trim_start_matches("$ cd ");
                if name == ".." {
                    break;
                } else {
                    let child = root.get_mut(name)
                               .ok_or_else(|| anyhow!("Invalid hierarchy ref: {}", name))?;
                    let Node::Dir {children, ..} = child else {
                        anyhow::bail!("Invalid hierarchy cast");
                    };

                    build_tree(lines, children)?;
                }
            } else if line.starts_with("$ ls") {
                // begin listing (ignore this case)
            } else if line.starts_with("dir ") {
                // dir entry
                let (_, name) = line.split_once(' ').ok_or_else(|| anyhow!("Invalid file line"))?;
                root.insert(name.to_owned(), Node::Dir {
                    total_size: 0,
                    children: HashMap::new()
                });
            } else {
                // file entry
                let (num, name) = line.split_once(' ').ok_or_else(|| anyhow!("Invalid file line"))?;
                let size = u64::from_str(num)?;
                root.insert(name.to_owned(), Node::File(size));
            }
        }

        Ok(())
    }

    let mut children = HashMap::new();
    let mut lines_iter = lines.iter().map(|s| s.as_str()).skip(1);
    build_tree(&mut lines_iter, &mut children)?;

    let mut out = Node::Dir {total_size: 0, children};
    out.update_sizes();

    Ok(out)
}

fn solve1(input: &Input) -> Result<u64> {
    let mut out = 0;
    input.on_dirs(&mut |size, _| {
        if size <= 100_000 {
            out += size;
        }
    });

    Ok(out)
}

fn solve2(input: &Input) -> Result<u64> {
    const TOTAL: u64 = 70_000_000;
    const NEEDED: u64 = 30_000_000;

    // how much do we need to free up?
    let Node::Dir {total_size: used, ..} = input else { unreachable!() };
    let to_free = NEEDED - (TOTAL - used);

    // find smallest dir to delete which fits the criteria
    let mut to_del: Option<u64> = None;
    input.on_dirs(&mut |size, _| {
        if size < to_free {
            return;
        }

        if let Some(old) = to_del.take() {
            to_del = Some(old.min(size));
        } else {
            to_del = Some(size);
        }
    });

    Ok(to_del.unwrap())
}

problem!(load_input => Node => (solve1, solve2));
