use anyhow::Result;

#[derive(Debug)]
enum Operand {
    Const(u64),
    Old,
}

#[derive(Debug)]
enum Op {
    Mul(Operand, Operand),
    Add(Operand, Operand),
}

impl Op {
    /// Apply the operation to the current worry level
    fn apply(&self, worry: u64) -> u64 {
        fn evaluate(op: &Operand, worry: u64) -> u64 {
            match *op {
                Operand::Old => worry,
                Operand::Const(n) => n,
            }
        }

        match self {
            Op::Mul(a, b) => evaluate(a, worry) * evaluate(b, worry),
            Op::Add(a, b) => evaluate(a, worry) + evaluate(b, worry),
        }
    }
}

type Item = u64;

#[derive(Debug)]
struct Monkey {
    items: Vec<Item>,

    /// Operation
    operation: Op,

    /// Divisibility test
    divisor: u64,

    /// Monkey ID to throw to if test is `(false, true)`
    branches: (usize, usize),
}

fn load_input(input: &mut dyn std::io::BufRead) -> Result<Input> {
    let mut data = String::new();
    input.read_to_string(&mut data)?;

    fn parse_operation(op: &str) -> Result<Op> {
        anyhow::ensure!(op.starts_with("new = "));
        let expr = op.trim_start_matches("new = ");
        let parts = expr.split(' ').collect::<Vec<_>>();

        anyhow::ensure!(parts.len() == 3);

        let lhs = match parts[0] {
            "old" => Operand::Old,
            n => Operand::Const(n.parse::<u64>()?),
        };
        let rhs = match parts[2] {
            "old" => Operand::Old,
            n => Operand::Const(n.parse::<u64>()?),
        };

        Ok(match parts[1] {
            "+" => Op::Add(lhs, rhs),
            "*" => Op::Mul(lhs, rhs),
            _ => anyhow::bail!("Invalid operator"),
        })
    }

    fn parse_monkey(desc: &str) -> Result<Monkey> {
        let lines = desc.lines().collect::<Vec<_>>();
        let [l_monkey, l_start, l_op, l_test, l_yes, l_no] = lines[..] else {
            anyhow::bail!("Invalid monkey definition");
        };

        anyhow::ensure!(l_monkey.starts_with("Monkey "), "Invalid monkey start line");
        anyhow::ensure!(l_start.starts_with("  Starting items: "), "Invalid monkey items line");
        anyhow::ensure!(l_op.starts_with("  Operation: "), "Invalid monkey operation line");
        anyhow::ensure!(l_test.starts_with("  Test: "), "Invalid monkey test line");
        anyhow::ensure!(l_yes.starts_with("    If true: "), "Invalid monkey true-branch line");
        anyhow::ensure!(l_no.starts_with("    If false: "), "Invalid monkey false-branch line");

        let items = l_start.trim_start_matches("  Starting items: ")
                           .split(", ")
                           .map(|item| item.parse::<u64>().map_err(|e| e.into()))
                           .collect::<Result<Vec<_>>>()?;
        let operation = parse_operation(l_op.trim_start_matches("  Operation: "))?;
        let divisor = l_test.trim_start_matches("  Test: divisible by ")
                            .parse::<u64>()?;
        let t_branch = l_yes.trim_start_matches("    If true: throw to monkey ").parse::<usize>()?;
        let f_branch = l_no.trim_start_matches("    If false: throw to monkey ").parse::<usize>()?;

        Ok(Monkey { items, operation, divisor, branches: (f_branch, t_branch) })
    }

    data.split("\n\n").map(parse_monkey).collect::<Result<Vec<_>>>()
}

fn lcm(xs: Vec<u64>) -> u64 {
    fn gcd(a: u64, b: u64) -> u64 {
        if b == 0 {
            a
        } else {
            gcd(b, a % b)
        }
    }

    let mut out = xs[0];
    for elem in xs {
        out = elem * out/gcd(elem, out);
    }

    out
}

/// Simulate monkey business
///
/// This simulation treats worry values as integers modulo k, where k is the LCM of all monkeys'
/// divisors.
struct Simulation<'i> {
    input: &'i Input,

    /// Whether to enable worry decay
    decay: bool,

    /// LCM value; worry is stored mod k.
    k: u64,

    /// The list of items held by each monkey
    items: Vec<Vec<Item>>,

    /// How many items each monkey inspected
    inspected: Vec<usize>,
}

impl<'i> Simulation<'i> {
    fn new(input: &'i Input, decay: bool) -> Self {
        Self {
            k: lcm(input.iter().map(|m| m.divisor).collect::<Vec<_>>()),
            items: input.iter().map(|m| m.items.clone()).collect(),
            inspected: vec![0; input.len()],
            decay, input,
        }
    }

    /// Execute a given monkey's turn
    fn step_turn(&mut self, index: usize) {
        assert!(index < self.items.len());
        let monkey = &self.input[index];
        let inspected = &mut self.inspected[index];

        // process each item on this monkey
        while !self.items[index].is_empty() {
            *inspected += 1;

            let mut item = self.items[index].remove(0);

            // apply operation
            item = monkey.operation.apply(item) % self.k;

            if self.decay {
                // apply worry decay
                item /= 3;
            }

            // test worry level
            let target = if item % monkey.divisor == 0 { monkey.branches.1 }
                         else { monkey.branches.0 };
            self.items[target].push(item);
        }
    }

    /// Step the simulation forwards by one round
    fn step_round(&mut self) {
        for m in 0..self.items.len() {
            self.step_turn(m);
        }
    }

    /// Return the level of monkey business in the current state
    fn monkey_business(&self) -> usize {
        let mut inspected = self.inspected.clone();
        inspected.sort_unstable();

        inspected[inspected.len()-1] * inspected[inspected.len()-2]
    }
}

fn solve1(input: &Input) -> Result<usize> {
    let mut sim = Simulation::new(input, true);
    for _ in 0..20 {
        sim.step_round();
    }

    Ok(sim.monkey_business())
}

fn solve2(input: &Input) -> Result<usize> {
    let mut sim = Simulation::new(input, false);
    for _ in 0..10_000 {
        sim.step_round();
    }

    Ok(sim.monkey_business())
}

problem!(load_input => Vec<Monkey> => (solve1, solve2));
