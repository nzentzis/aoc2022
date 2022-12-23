use anyhow::Result;

use std::rc::Rc;

#[derive(Clone, Debug)]
enum Expr {
    /// Variable node
    V(i64),

    C(i64),
    Add(Rc<Self>, Rc<Self>),
    Sub(Rc<Self>, Rc<Self>),
    Mul(Rc<Self>, Rc<Self>),
    Div(Rc<Self>, Rc<Self>),
    Equal(Rc<Self>, Rc<Self>),
}

impl Expr {
    fn make_equal(&self) -> Option<Self> {
        match self {
            Self::C(_) => None,
            Self::V(_) => None,
            Self::Add(l,r) => Some(Self::Equal(Rc::clone(l), Rc::clone(r))),
            Self::Sub(l,r) => Some(Self::Equal(Rc::clone(l), Rc::clone(r))),
            Self::Mul(l,r) => Some(Self::Equal(Rc::clone(l), Rc::clone(r))),
            Self::Div(l,r) => Some(Self::Equal(Rc::clone(l), Rc::clone(r))),
            Self::Equal(l,r) => Some(Self::Equal(Rc::clone(l), Rc::clone(r))),
        }
    }

    fn is_const(&self) -> bool {
        match self {
            Self::C(_) => true,
            Self::V(_) => false,
            Self::Add(l, r) => l.is_const() && r.is_const(),
            Self::Sub(l, r) => l.is_const() && r.is_const(),
            Self::Mul(l, r) => l.is_const() && r.is_const(),
            Self::Div(l, r) => l.is_const() && r.is_const(),
            Self::Equal(l, r) => l.is_const() && r.is_const(),
        }
    }

    fn eval(&self, var: Option<i64>) -> i64 {
        match self {
            Self::C(x) => *x,
            Self::V(x) => var.unwrap_or(*x),
            Self::Add(l, r) => l.eval(var) + r.eval(var),
            Self::Sub(l, r) => l.eval(var) - r.eval(var),
            Self::Mul(l, r) => l.eval(var) * r.eval(var),
            Self::Div(l, r) => l.eval(var) / r.eval(var),
            Self::Equal(l, r) => (l.eval(var) == r.eval(var)) as i64,
        }
    }

    fn const_fold(&mut self) {
        use std::ops::Deref;

        match self {
            Self::C(_) => {},
            Self::V(_) => {},
            Self::Add(l, r) => {
                if let (Self::C(l), Self::C(r)) = ((*l).deref(), (*r).deref()) {
                    *self = Self::C(l + r);
                }
            }
            Self::Sub(l, r) => {
                if let (Self::C(l), Self::C(r)) = ((*l).deref(), (*r).deref()) {
                    *self = Self::C(l - r);
                }
            }
            Self::Mul(l, r) => {
                if let (Self::C(l), Self::C(r)) = ((*l).deref(), (*r).deref()) {
                    *self = Self::C(l * r);
                }
            }
            Self::Div(l, r) => {
                if let (Self::C(l), Self::C(r)) = ((*l).deref(), (*r).deref()) {
                    *self = Self::C(l / r);
                }
            }
            Self::Equal(l, r) => {
                if let (Self::C(l), Self::C(r)) = ((*l).deref(), (*r).deref()) {
                    *self = Self::C((l == r) as i64);
                }
            }
        }
    }

    /// Find a variable value which will make this expression take a specified value
    ///
    /// If this isn't possible, or if there are multiple values which satisfy the constraint,
    /// return `None`.
    fn solve_for(&self, tgt: i64) -> Option<i64> {
        match self {
            Self::C(_) => None, // either zero or infinite solutions
            Self::V(_) => Some(tgt),
            Self::Add(l, r) => {
                match (l.is_const(), r.is_const()) {
                    (false, false) => None, // unable to solve this case currently
                    (true, false) => r.solve_for(tgt - l.eval(None)),
                    (false, true) => l.solve_for(tgt - r.eval(None)),
                    (true, true) => None, // either zero or infinite solutions
                }
            },
            Self::Sub(l, r) => {
                match (l.is_const(), r.is_const()) {
                    (false, false) => None, // unable to solve this case currently
                    (true, false) => r.solve_for(l.eval(None) - tgt),
                    (false, true) => l.solve_for(tgt + r.eval(None)),
                    (true, true) => None, // either zero or infinite solutions
                }
            },
            Self::Mul(l, r) => {
                match (l.is_const(), r.is_const()) {
                    (false, false) => None, // unable to solve this case currently
                    (true, false) => r.solve_for(tgt / l.eval(None)),
                    (false, true) => l.solve_for(tgt / r.eval(None)),
                    (true, true) => None, // either zero or infinite solutions
                }
            },
            Self::Div(l, r) => {
                match (l.is_const(), r.is_const()) {
                    (false, false) => None, // unable to solve this case currently
                    (true, false) => r.solve_for(l.eval(None) / tgt),
                    (false, true) => l.solve_for(tgt * r.eval(None)),
                    (true, true) => None, // either zero or infinite solutions
                }
            },
            Self::Equal(l, r) => {
                if tgt != 1 {
                    return None;
                }

                match (l.is_const(), r.is_const()) {
                    (false, false) => None, // unable to solve this case currently
                    (true, false) => r.solve_for(l.eval(None)),
                    (false, true) => l.solve_for(r.eval(None)),
                    (true, true) => None, // either zero or infinite solutions
                }
            },
        }
    }
}

fn load_input(input: &mut dyn std::io::BufRead) -> Result<Input> {
    enum Val {
        V(i64),
        C(i64),
        Op(String, char, String),
    }

    struct Line {
        name: String,
        val: Val,
    }

    let lines = crate::util::read_lines_regex(
        input, r#"^([a-z]{4}): (\d+|[a-z]{4} [-+*/] [a-z]{4})$"#,
        |caps| {
            let name = caps.get(1).unwrap().as_str().to_owned();
            let value = caps.get(2).unwrap().as_str();

            let value = match value.parse() {
                Ok(n) => Val::C(n),
                Err(_) => {
                    let l = value[..4].to_owned();
                    let r = value[7..].to_owned();
                    let op = value[5..6].chars().next().unwrap();

                    Val::Op(l, op, r)
                }
            };

            Ok(Line { name, val: value })
        }
    )?;

    // map name to expr values
    let mut name_map = lines.into_iter()
                      .map(|Line {name, val}| (name, val))
                      .collect::<std::collections::HashMap<_,_>>();

    // add human variable
    let humn = name_map.get("humn").ok_or_else(|| anyhow::anyhow!("No human"))?;
    let Val::C(humn) = humn else { anyhow::bail!("Human node must be constant") };
    *name_map.get_mut("humn").unwrap() = Val::V(*humn);

    let mut exprs = std::collections::HashMap::new();
    let mut pending = name_map.into_iter().collect::<std::collections::VecDeque<_>>();
    while let Some((name, val)) = pending.pop_front() {
        let mut expr = match val {
            Val::C(x) => Expr::C(x),
            Val::V(x) => Expr::V(x),
            Val::Op(lhs, op, rhs) => {
                let (l, r) = match exprs.get(&lhs).zip(exprs.get(&rhs)) {
                    Some((l, r)) => (Rc::clone(l), Rc::clone(r)),
                    None => {
                        // one of the args isn't ready yet - process this later
                        pending.push_back((name, Val::Op(lhs, op, rhs)));
                        continue;
                    }
                };

                match op {
                    '+' => Expr::Add(l, r),
                    '-' => Expr::Sub(l, r),
                    '*' => Expr::Mul(l, r),
                    '/' => Expr::Div(l, r),
                    _ => anyhow::bail!("Invalid operator"),
                }
            }
        };
        expr.const_fold();
        exprs.insert(name, Rc::new(expr));
    }

    // resolve root/humn element
    let root = exprs.get("root").ok_or_else(|| anyhow::anyhow!("No root monkey"))?;

    Ok(Rc::clone(root))
}

fn solve1(input: &Input) -> Result<i64> {
    Ok(input.eval(None))
}

fn solve2(input: &Input) -> Result<i64> {
    let modified = input.make_equal()
                  .ok_or_else(|| anyhow::anyhow!("Unable to build equality monkey"))?;
    modified.solve_for(1).ok_or_else(|| anyhow::anyhow!("Unable to solve equation"))
}

problem!(load_input => Rc<Expr> => (solve1, solve2));
