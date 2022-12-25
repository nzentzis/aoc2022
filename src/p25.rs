use anyhow::Result;

type Digit = i8;

#[derive(Debug, PartialEq, Eq)]
struct Number {
    /// Digits, in order of increasing significance
    digits: Vec<Digit>,
}

impl std::str::FromStr for Number {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut digits = s.chars()
                        .map(|c| match c {
                            '2' => Ok(2),
                            '1' => Ok(1),
                            '0' => Ok(0),
                            '-' => Ok(-1),
                            '=' => Ok(-2),
                            _   => Err(anyhow::anyhow!("Invalid digit"))
                        })
                        .collect::<Result<Vec<_>>>()?;
        digits.reverse();
        Ok(Self { digits })
    }
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for d in self.digits.iter().rev() {
            let c = match d {
                2 => '2',
                1 => '1',
                0 => '0',
                -1 => '-',
                -2 => '=',
                _ => panic!("Invalid number")
            };
            write!(f, "{}", c)?;
        }
        Ok(())
    }
}

impl Number {
    fn to_normal(&self) -> i64 {
        let mut s = 0;
        for (i, d) in self.digits.iter().cloned().enumerate() {
            s += (d as i64) * 5i64.pow(i as u32);
        }
        s
    }

    fn from_number(mut n: i64) -> Self {
        // first, convert to base 5
        let mut digits = Vec::new();
        while n > 4 {
            digits.push((n % 5) as i8);
            n /= 5;
        }
        digits.push(n as i8);

        // walk to the left, replacing 3 with (1, -2) and 4 with (1, -1)
        let mut accum = 0;
        for x in digits.iter_mut() {
            match *x + accum {
                5 => {
                    *x = 0;
                    accum = 1;
                }
                4 => {
                    *x = -1;
                    accum = 1;
                }
                3 => {
                    *x = -2;
                    accum = 1;
                }
                v => {
                    *x = v;
                    accum = 0;
                }
            }
        }
        if accum != 0 {
            digits.push(accum);
        }

        Self { digits }
    }
}

#[test]
fn test_from_number() {
    assert_eq!(Number::from_number(1257),
               Number {digits: vec![2,1,0,0,2]});
    assert_eq!(Number::from_number(1747),
               Number {digits: vec![2,-1,0,-1,-2,1]});
}

fn solve1(input: &Input) -> Result<String> {
    let s = input.iter().map(|n| n.to_normal()).sum::<i64>();
    Ok(Number::from_number(s).to_string())
}

problem!(crate::util::load_lines => Vec<Number> => (solve1));
