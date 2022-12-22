use anyhow::{anyhow, Result};

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
struct Item(u8);

impl Item {
    fn priority(&self) -> usize {
        (self.0 as usize) + 1
    }
}

impl TryFrom<char> for Item {
    type Error = anyhow::Error;

    fn try_from(c: char) -> Result<Self> {
        Ok(match c {
            'a'..='z' => Self((c as u8) - b'a'),
            'A'..='Z' => Self(26 + ((c as u8) - b'A')),
            _ => anyhow::bail!("Invalid sack character '{}'", c),
        })
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
struct ItemSet(u64);

impl ItemSet {
    fn single(item: Item) -> u64 {
        1 << item.0
    }

    fn from_items(items: &[Item]) -> Self {
        let mut out = 0u64;
        for bit in items.iter().cloned().map(Self::single) {
            out |= bit;
        }

        Self(out)
    }

    fn intersect(&self, other: &Self) -> Self {
        Self(self.0 & other.0)
    }

    /// If only one item exists in this set, return it
    fn only_item(&self) -> Option<Item> {
        if self.0.count_ones() == 1 {
            Some(Item(self.0.trailing_zeros() as u8))
        } else {
            None
        }
    }
}

struct Rucksack {
    items: Vec<Item>,
}

impl Rucksack {
    /// Split the container into a slice for each compartment
    fn compartments(&self) -> (&[Item], &[Item]) {
        assert_eq!(self.items.len() % 2, 0);
        self.items.as_slice().split_at(self.items.len() / 2)
    }

    /// Find the misplaced item using the bitset method
    fn misplaced_item(&self) -> Option<Item> {
        let (left, right) = self.compartments();
        ItemSet::from_items(left)
                .intersect(&ItemSet::from_items(right))
                .only_item()
    }
}

fn load_input(input: &mut dyn std::io::BufRead) -> Result<Input> {
    crate::util::read_lines(input, |line| {
        let items = line.chars().map(Item::try_from).collect::<Result<_>>()?;
        Ok(Rucksack { items })
    })
}

fn solve1(input: &Input) -> Result<usize> {
    Ok(input.iter()
            .map(|sack| sack.misplaced_item().expect("No misplaced item found").priority())
            .sum())
}

fn solve2(input: &Input) -> Result<usize> {
    let mut out = 0;
    for group in input.chunks(3) {
        anyhow::ensure!(group.len() == 3);
        let badge = ItemSet::from_items(&group[0].items)
                   .intersect(&ItemSet::from_items(&group[1].items))
                   .intersect(&ItemSet::from_items(&group[2].items))
                   .only_item()
                   .ok_or_else(|| anyhow!("No single badge item found"))?;

        out += badge.priority();
    }

    Ok(out)
}

problem!(load_input => Vec<Rucksack> => (solve1, solve2));
