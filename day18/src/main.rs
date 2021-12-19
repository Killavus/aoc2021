use std::collections::{HashMap, VecDeque};
use std::fmt::Display;
use std::fs;
use std::str::Chars;

#[derive(Debug, Clone, Copy)]
enum Element {
    Pair(usize, usize),
    Value(usize),
}

#[derive(Debug)]

struct Explode(usize, usize);

impl Element {
    fn parse(mut storage: Vec<Self>, s: &str) -> (usize, HashMap<usize, usize>, Vec<Self>) {
        let mut parent = HashMap::new();
        let (idx, _) = Self::parse_inner(&mut storage, &mut parent, s.chars());

        (idx, parent, storage)
    }

    fn set_left(&mut self, new_l: usize) {
        if let Element::Pair(l, _) = self {
            *l = new_l;
        } else {
            panic!("trying to take left of value");
        }
    }

    fn set_right(&mut self, new_r: usize) {
        if let Element::Pair(_, r) = self {
            *r = new_r;
        } else {
            panic!("trying to take right of value");
        }
    }

    fn add_value(&mut self, new_v: usize) -> usize {
        if let Element::Value(n) = self {
            *n += new_v;
            *n
        } else {
            panic!("trying to take value of pair");
        }
    }

    fn value(&self) -> usize {
        if let Element::Value(n) = self {
            *n
        } else {
            panic!("trying to take value of pair");
        }
    }

    fn left(&self) -> usize {
        if let Element::Pair(l, _) = self {
            *l
        } else {
            panic!("trying to take left of value");
        }
    }

    fn right(&self) -> usize {
        if let Element::Pair(_, r) = self {
            *r
        } else {
            panic!("trying to take right of value");
        }
    }

    fn parse_inner<'str>(
        storage: &mut Vec<Self>,
        parent: &mut HashMap<usize, usize>,
        mut iter: Chars<'str>,
    ) -> (usize, Chars<'str>) {
        let char = iter.next().unwrap();

        match char {
            '[' => {
                let (l_idx, mut iter) = Self::parse_inner(storage, parent, iter);
                iter.next().unwrap();
                let (r_idx, mut iter) = Self::parse_inner(storage, parent, iter);
                storage.push(Element::Pair(l_idx, r_idx));
                iter.next().unwrap();
                *parent.entry(l_idx).or_insert(0) = storage.len() - 1;
                *parent.entry(r_idx).or_insert(0) = storage.len() - 1;
                (storage.len() - 1, iter)
            }
            ',' => Self::parse_inner(storage, parent, iter),
            c => {
                let digit = c.to_digit(10).unwrap() as usize;
                storage.push(Element::Value(digit));
                (storage.len() - 1, iter)
            }
        }
    }

    fn represent(
        &self,
        storage: &[Element],
        f: &mut std::fmt::Formatter<'_>,
        highlight: &Option<usize>,
    ) {
        use Element::*;

        match self {
            Value(n) => {
                write!(f, "{}", n).ok();
            }
            Pair(l, r) => {
                write!(f, "[").ok();
                if let Some(idx) = highlight {
                    if *l == *idx {
                        write!(f, "*").ok();
                    }
                }
                storage[*l].represent(storage, f, highlight);
                if let Some(idx) = highlight {
                    if *l == *idx {
                        write!(f, "*").ok();
                    }
                }
                write!(f, ",").ok();
                if let Some(idx) = highlight {
                    if *r == *idx {
                        write!(f, "*").ok();
                    }
                }
                storage[*r].represent(storage, f, highlight);
                if let Some(idx) = highlight {
                    if *r == *idx {
                        write!(f, "*").ok();
                    }
                }
                write!(f, "]").ok();
            }
        }
    }

    fn magnitude(&self, storage: &[Element]) -> usize {
        use Element::*;

        match self {
            Value(n) => *n,
            Pair(l, r) => 3 * storage[*l].magnitude(&storage) + 2 * storage[*r].magnitude(&storage),
        }
    }
}

#[derive(Debug, Clone)]
struct Number {
    storage: Vec<Element>,
    parent: HashMap<usize, usize>,
    root: usize,
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let current = &self.storage[self.root];
        current.represent(&self.storage, f, &None);
        Ok(())
    }
}

struct NumberView<'num>(&'num Number, usize);

impl<'num> Display for NumberView<'num> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let current = &self.0.storage[self.0.root];
        current.represent(&self.0.storage, f, &Some(self.1));
        Ok(())
    }
}

impl Number {
    fn add(mut self, other: &Number) -> Self {
        let idx_offset = self.storage.len();
        self.storage
            .push(Element::Pair(self.root, other.root + idx_offset + 1));
        self.storage.extend(other.storage.clone());

        self.parent.extend(
            other
                .parent
                .iter()
                .map(|(k, v)| (*k + idx_offset + 1, *v + idx_offset + 1)),
        );

        *self.parent.entry(self.root).or_insert(0) = idx_offset;
        *self.parent.entry(other.root + idx_offset + 1).or_insert(0) = idx_offset;

        for elem in self.storage.iter_mut().skip(idx_offset + 1) {
            match elem {
                Element::Pair(l, r) => {
                    *l += idx_offset + 1;
                    *r += idx_offset + 1;
                }
                _ => {}
            }
        }

        self.root = idx_offset;
        let mut actions = VecDeque::new();
        self.reduce(self.root, 0, &mut actions);
        self.apply_reduce(actions);

        self
    }

    fn first_on_left(&self, start: usize, idx: usize, depth: usize) -> Option<(usize, usize)> {
        let mut candidate = start;
        if let Element::Pair(l, _) = &self.storage[start] {
            if *l == idx {
                if self.parent.get(&start).is_some() {
                    return self.first_on_left(self.parent[&start], start, depth - 1);
                } else {
                    return None;
                }
            } else {
                candidate = *l;
            }
        }

        let mut depth_now = depth + 1;

        while let Element::Pair(_, r) = &self.storage[candidate] {
            candidate = *r;
            depth_now += 1;
        }

        Some((candidate, depth_now))
    }

    fn first_on_right(&self, start: usize, idx: usize, depth: usize) -> Option<(usize, usize)> {
        let mut candidate = start;
        if let Element::Pair(_, r) = &self.storage[start] {
            if *r == idx {
                if self.parent.get(&start).is_some() {
                    return self.first_on_right(self.parent[&start], start, depth - 1);
                } else {
                    return None;
                }
            } else {
                candidate = *r;
            }
        }

        let mut depth_now = depth + 1;

        while let Element::Pair(l, _) = &self.storage[candidate] {
            candidate = *l;
            depth_now += 1;
        }

        Some((candidate, depth_now))
    }

    fn find_split(&self, idx: usize, depth: usize) -> Option<(usize, usize)> {
        match &self.storage[idx] {
            Element::Value(n) => {
                if *n > 9 {
                    Some((idx, depth))
                } else {
                    None
                }
            }
            Element::Pair(l, r) => self
                .find_split(*l, depth + 1)
                .or_else(|| self.find_split(*r, depth + 1)),
        }
    }

    fn apply_reduce(&mut self, mut explosions: VecDeque<Explode>) {
        loop {
            let mut any_operation = false;
            while let Some(Explode(idx, depth)) = explosions.pop_front() {
                if self.storage[self.parent[&idx]].left() != idx
                    && self.storage[self.parent[&idx]].right() != idx
                {
                    continue;
                }

                any_operation = true;
                let pair = self.storage[idx].clone();
                let parent_idx = self.parent[&idx];

                let exploded_lv = self.storage[pair.left()].value();
                let exploded_rv = self.storage[pair.right()].value();

                if let Some((left, _)) = self.first_on_left(parent_idx, idx, depth - 1) {
                    self.storage[left].add_value(exploded_lv);
                }

                if let Some((right, _)) = self.first_on_right(parent_idx, idx, depth - 1) {
                    self.storage[right].add_value(exploded_rv);
                }

                let elem_idx = self.storage.len();
                self.storage.push(Element::Value(0));
                self.parent.entry(elem_idx).or_insert(parent_idx);

                if self.storage[parent_idx].left() == idx {
                    self.storage[parent_idx].set_left(elem_idx);
                } else {
                    self.storage[parent_idx].set_right(elem_idx);
                }
            }

            while let Some((idx, depth)) = self.find_split(self.root, 0) {
                any_operation = true;
                let value = self.storage[idx].value();
                let left_v = value / 2;
                let right_v = value / 2 + if value % 2 == 0 { 0 } else { 1 };

                let idx_offset = self.storage.len();
                self.storage.push(Element::Value(left_v));
                self.storage.push(Element::Value(right_v));
                self.storage.push(Element::Pair(idx_offset, idx_offset + 1));

                let parent_idx = self.parent[&idx];
                self.parent.entry(idx_offset).or_insert(idx_offset + 2);
                self.parent.entry(idx_offset + 1).or_insert(idx_offset + 2);
                self.parent.entry(idx_offset + 2).or_insert(parent_idx);

                if self.storage[parent_idx].left() == idx {
                    self.storage[parent_idx].set_left(idx_offset + 2);
                } else {
                    self.storage[parent_idx].set_right(idx_offset + 2);
                }

                if depth == 4 {
                    explosions.push_front(Explode(idx_offset + 2, depth));
                    break;
                }
            }

            if !any_operation {
                break;
            }
        }
    }

    fn magnitude(&self) -> usize {
        self.storage[self.root].magnitude(&self.storage)
    }

    fn reduce(&self, idx: usize, depth: usize, explosions: &mut VecDeque<Explode>) {
        match &self.storage[idx] {
            Element::Pair(left, right) => {
                if depth == 4 {
                    explosions.push_back(Explode(idx, depth));
                } else {
                    self.reduce(*left, depth + 1, explosions);
                    self.reduce(*right, depth + 1, explosions);
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let s = fs::read_to_string("./input").unwrap();
    let numbers = s
        .lines()
        .map(|line| Element::parse(vec![], line))
        .map(|(root, parent, storage)| Number {
            storage,
            parent,
            root,
        })
        .collect::<Vec<_>>();

    let mut iter = numbers.clone().into_iter();
    let number = iter.next().unwrap();
    let number = iter.fold(number, |n, other_number| n.add(&other_number));

    println!(
        "Number after all additions is: {}, magnitude: {}",
        number,
        number.magnitude()
    );

    let mut max_magnitude = 0;
    for i in 0..numbers.len() {
        for j in 0..numbers.len() {
            if i == j {
                continue;
            }

            let magnitude = numbers[i].clone().add(&numbers[j]).magnitude();

            if magnitude > max_magnitude {
                max_magnitude = magnitude;
            }
        }
    }

    println!(
        "Maximum magnitude from adding two numbers only is {}",
        max_magnitude
    );
}
