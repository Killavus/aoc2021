use anyhow::anyhow;
use std::{cell::RefCell, fmt::Display, rc::Rc, str::FromStr};

#[derive(Debug)]
pub struct Chain {
    storage: Rc<RefCell<Vec<char>>>,
    index: usize,
    next: Option<Box<Chain>>,
}

impl FromStr for Box<Chain> {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Chain::from_chars(s.chars().collect())
            .ok_or(anyhow!("failed to generate chain from an empty string"))?)
    }
}

impl Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut current_letter = self.get();
        let mut current = self.next_immutable();
        write!(f, "{}", current_letter)?;
        while let Some(elem) = current.take() {
            current_letter = elem.get();
            current = self.next_immutable();
            write!(f, "{}", current_letter)?;
        }

        Ok(())
    }
}

impl Chain {
    pub fn from_chars(v: Vec<char>) -> Option<Box<Self>> {
        let size = v.len();
        let v_refcounted = Rc::new(RefCell::new(v));

        let mut first: Option<Box<Self>> = None;
        let mut current: Option<&mut Self> = None;

        for i in 0..size {
            match current.take() {
                Some(link) => {
                    link.next = Some(Box::new(Self {
                        storage: v_refcounted.clone(),
                        index: i,
                        next: None,
                    }));

                    current = link.next.as_mut().map(AsMut::as_mut);
                }
                None => {
                    first = Some(Box::new(Self {
                        storage: v_refcounted.clone(),
                        index: 0,
                        next: None,
                    }));

                    current = first.as_mut().map(AsMut::as_mut);
                }
            }
        }

        first
    }

    pub fn get(&self) -> char {
        self.storage.borrow()[self.index]
    }

    pub fn push_after(&mut self, elem: char) -> Option<&mut Self> {
        let prev_next = self.next.take();
        let mut borrow = self.storage.borrow_mut();
        borrow.push(elem);
        self.next = Some(Box::new(Self {
            storage: self.storage.clone(),
            index: borrow.len() - 1,
            next: prev_next,
        }));

        self.next.as_mut().and_then(|chain| chain.next())
    }

    pub fn next(&mut self) -> Option<&mut Self> {
        self.next.as_mut().map(AsMut::as_mut)
    }

    pub fn next_immutable(&self) -> Option<&Self> {
        self.next.as_ref().map(AsRef::as_ref)
    }
}
