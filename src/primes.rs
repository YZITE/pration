use once_cell::sync::Lazy;
use std::sync::RwLock;

pub static PRIMES: Lazy<RwLock<Store>> = Lazy::new(|| RwLock::new(Store::new()));

type Prime = u64;

#[derive(Clone)]
pub struct Store {
    inner: Vec<Prime>,
}

pub struct Iter<'a> {
    inner: &'a mut Store,
    pos: Option<usize>,
}

impl Store {
    // SAFETY: must be non-public
    fn new() -> Self {
        Self {
            // for some baseline efficiency, cache the prime numbers between 1 and 100
            #[rustfmt::skip]
            inner: vec![
                2, 3, 5, 7,
                11, 13, 17, 19,
                23, 29, 31, 37,
                41, 43, 47, 53, 59,
                61, 67, 71, 73, 79,
                83, 89, 97,
            ],
        }
    }

    pub fn get(&self) -> &[Prime] {
        &self.inner[..]
    }

    // may panic, although runtime -> inf. beforehand
    pub fn find_next(&mut self) {
        let mut t = *self.inner.last().unwrap();
        let inwo2 = self.inner[1..].iter();
        // this is really slow
        // TODO: low-hanging fruit, make this faster
        loop {
            t += 2;
            if t > Prime::MAX - 3 {
                panic!("primes reached the maximum");
            }
            if !inwo2
                .clone()
                .copied()
                .take_while(|&i| (i * i) <= t)
                .any(|i| t % i == 0)
            {
                break;
            }
        }
        let _ = inwo2;
        self.inner.push(t);
    }

    pub fn get_at(&mut self, n: usize) -> Prime {
        while self.inner.len() < n {
            self.find_next();
        }
        self.inner[n]
    }

    pub fn iter(&mut self) -> Iter<'_> {
        Iter {
            inner: self,
            pos: Some(0),
        }
    }
}

impl Iterator for Iter<'_> {
    type Item = Prime;
    fn next(&mut self) -> Option<Prime> {
        if let Some(x) = self.pos {
            if let Some(&y) = self.inner.inner.get(x) {
                self.pos = Some(x + 1);
                return Some(y);
            } else {
                self.pos = None;
            }
        }
        assert_eq!(self.pos, None);
        self.inner.find_next();
        self.inner.inner.last().copied()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn primes1000() {
        // we don't want this test to hide regressions in error handling in
        // other parts of the code
        let mut ps = crate::PRIMES.read().unwrap().clone();
        (0..10_000).for_each(|_| ps.find_next());
        for (n, i) in ps.inner.iter().enumerate().skip(1) {
            for j in &ps.inner[..n] {
                if i % j == 0 {
                    panic!("corruption found {} @ {} % {}", n, i, j);
                }
            }
        }
    }
}
