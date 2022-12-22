use anyhow::{Result, anyhow};

fn load_input(input: &mut dyn std::io::BufRead) -> Result<Input> {
    let mut data = String::new();
    input.read_to_string(&mut data)?;

    fn parse_packet<I: Iterator<Item=char>>(chars: &mut std::iter::Peekable<I>) -> Result<Packet> {
        match chars.next().ok_or_else(|| anyhow!("Invalid packet"))? {
            '[' => {
                let mut items = Vec::new();
                loop {
                    match chars.peek() {
                        Some(',') => {
                            chars.next();
                        }
                        Some(']') => {
                            chars.next();
                            break;
                        }
                        _ => {
                            items.push(parse_packet(chars)?);
                        }
                    }
                }

                Ok(Packet::L(items))
            },
            n => {
                let mut n = n.to_digit(10).ok_or_else(|| anyhow!("Invalid packet digit"))?;
                loop {
                    match chars.peek().ok_or_else(|| anyhow!("Invalid packet format"))? {
                        c if c.is_ascii_digit() => {
                            let m = c.to_digit(10).unwrap();
                            n = (n*10) + m;
                            chars.next();
                        }
                        _ => {
                            break;
                        }
                    }
                }

                Ok(Packet::N(n as u8))
            },
        }
    }

    data.split("\n\n")
        .map(|group| {
            let xs = group.split_once('\n').ok_or_else(|| anyhow!("Invalid packet group"))?;
            Ok((parse_packet(&mut xs.0.chars().peekable())?,
                parse_packet(&mut xs.1.chars().peekable())?))
        })
        .collect::<Result<Vec<_>>>()
}

#[derive(Debug, PartialEq, Eq)]
enum Packet {
    L(Vec<Self>),
    N(u8),
}

impl std::cmp::PartialOrd for Packet {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for Packet {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        match (self, other) {
            (Self::N(l), Self::N(r)) => l.cmp(r),
            (Self::N(l), r@Self::L(_)) => Self::L(vec![Self::N(*l)]).cmp(r),
            (l@Self::L(_), Self::N(r)) => l.cmp(&Self::L(vec![Self::N(*r)])),
            (Self::L(lhs), Self::L(rhs)) => {
                let mut i = 0;
                loop {
                    match (lhs.get(i), rhs.get(i)) {
                        (Some(l), Some(r)) => {
                            let o = l.cmp(r);
                            if o != Ordering::Equal {
                                return o;
                            }
                        }
                        (None, None) => { return Ordering::Equal; }
                        (None, Some(_)) => { return Ordering::Less; }
                        (Some(_), None) => { return Ordering::Greater; }
                    }
                    i += 1;
                }
            }
        }
    }
}

fn solve1(input: &Input) -> Result<usize> {
    Ok(input.iter()
            .enumerate()
            .filter(|(_, (a, b))| a.cmp(b) == std::cmp::Ordering::Less)
            .map(|(idx, _)| idx + 1)
            .sum())
}

fn solve2(input: &Input) -> Result<usize> {
    let d0 = Packet::L(vec![Packet::L(vec![Packet::N(2)])]);
    let d1 = Packet::L(vec![Packet::L(vec![Packet::N(6)])]);
    let mut packets = input.iter()
                     .flat_map(|(a, b)| [a, b])
                     .collect::<Vec<&Packet>>();
    packets.push(&d0);
    packets.push(&d1);
    packets.sort_unstable();

    let loc_d0 = packets.binary_search(&&d0).unwrap() + 1;
    let loc_d1 = packets.binary_search(&&d1).unwrap() + 1;

    Ok(loc_d0 * loc_d1)
}

problem!(load_input => Vec<(Packet, Packet)> => (solve1, solve2));
