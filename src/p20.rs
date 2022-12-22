use anyhow::Result;

use std::ptr;

struct ShiftNode<T> {
    value: T,

    prev: *mut Self,
    next: *mut Self,
}

struct ShiftList<T> {
    nodes: Vec<*mut ShiftNode<T>>,
    len: usize,
}

impl<T> ShiftList<T> {
    fn new(xs: Vec<T>) -> Self {
        // construct initial node list
        let mut nodes = xs.into_iter().map(|x| Box::into_raw(Box::new(ShiftNode {
            value: x,
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
        }))).collect::<Vec<_>>();

        // populate links
        let n = nodes.len();
        unsafe {
            // populate 2^0 links
            (*nodes[0]).prev = nodes[n-1];
            (*nodes[0]).next = nodes[1];
            (*nodes[n-1]).prev = nodes[n-2];
            (*nodes[n-1]).next = nodes[0];

            for i in 1..(nodes.len()-1) {
                (*nodes[i]).prev = nodes[i-1];
                (*nodes[i]).next = nodes[i+1];
            }
        }

        Self { nodes, len: n, }
    }

    fn get_node(&self, idx: usize) -> &T {
        unsafe {
            &(*self.nodes[idx]).value
        }
    }

    fn shift_node(&mut self, idx: usize, delta: isize) {
        unsafe {
            let node = self.nodes[idx];

            if delta > 0 {
                // remove ourselves from the linking structure
                let mut prev = (*self.nodes[idx]).prev;
                let next = (*self.nodes[idx]).next;

                (*prev).next = next;
                (*next).prev = prev;

                // advance N times forwards
                for _ in 0..delta {
                    prev = (*prev).next;
                }

                // re-insert
                let next = (*prev).next;
                (*prev).next = node;
                (*next).prev = node;
                (*node).next = next;
                (*node).prev = prev;
            } else if delta < 0 {
                // remove ourselves from the linking structure
                let prev = (*self.nodes[idx]).prev;
                let mut next = (*self.nodes[idx]).next;

                (*prev).next = next;
                (*next).prev = prev;

                // advance N times backwards
                for _ in 0..(-delta) {
                    next = (*next).prev;
                }

                // re-insert
                let prev = (*next).prev;
                (*prev).next = node;
                (*next).prev = node;
                (*node).next = next;
                (*node).prev = prev;
            }
        }
    }

    fn into_vec(self, first: usize) -> Vec<T> {
        let mut out = Vec::with_capacity(self.len);

        // start at the given element and unravel the chain until we loop
        let mut node = self.nodes[first];
        let first = node;

        unsafe {
            let b = Box::from_raw(node);
            out.push(b.value);
            node = b.next;

            while node != first {
                let b = Box::from_raw(node);
                out.push(b.value);
                node = b.next;
            }
        }

        out
    }
}

fn mix_sequence(seq: &[i64], k: usize) -> Vec<i64> {
    let n = seq.len();
    let zero_idx = seq.iter().position(|x| *x == 0).unwrap();
    let mut seq: ShiftList<_> = ShiftList::new(seq.to_owned());

    for _ in 0..k {
        for i in 0..n {
            let mut delta = *seq.get_node(i);
            if delta > 0 {
                delta = delta % (n as i64 - 1);
            } else if delta < 0 {
                delta = -((-delta) % (n as i64 - 1));
            }
            seq.shift_node(i, delta as isize);
        }
    }

    seq.into_vec(zero_idx)
}

fn solve1(input: &Input) -> Result<i64> {
    let res = mix_sequence(input, 1);
    Ok(res[1000 % res.len()] +
        res[2000 % res.len()] +
        res[3000 % res.len()])
}

fn solve2(input: &Input) -> Result<i64> {
    let mut data = input.to_owned();
    for x in &mut data {
        *x *= 811589153;
    }

    let res = mix_sequence(&data, 10);
    Ok(res[1000 % res.len()] +
        res[2000 % res.len()] +
        res[3000 % res.len()])
}

problem!(crate::util::load_lines => Vec<i64> => (solve1, solve2));
