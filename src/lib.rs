// NOTE: we might want to use unsafe code
#![forbid(
    clippy::cast_ptr_alignment,
    trivial_casts,
    unconditional_recursion
)]

pub mod primes;

trait_set::trait_set! {
    pub trait WholeNum = num_traits::int::PrimInt + num_traits::sign::Signed;
}

/// non-zero positive rational number
/// NOTE: by default 1
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Urat<E: WholeNum> {
    // values correspond to the entries in PRIMES.
    // it is ok to serialize them independently of PRIMES,
    // because PRIMES should always yield the same values per position.
    inner: Vec<E>,
}

impl<E: WholeNum> Default for Urat<E> {
    #[inline]
    fn default() -> Self {
        Self { inner: vec![] }
    }
}

impl<E: WholeNum> Urat<E> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    fn reserve(&mut self, upto_excl: usize) {
        if self.inner.len() < upto_excl {
            self.inner.resize(upto_excl, E::zero());
        }
    }

    fn reduce(&mut self) {
        let zero = E::zero();
        self.inner.truncate(
            self.inner.len()
                - self
                    .inner
                    .iter()
                    .rev()
                    .take_while(move |&i| i == &zero)
                    .count(),
        );
    }

    fn rfr<F: FnMut((&mut E, &E))>(&mut self, rhs: &[E], f: F) {
        self.reserve(rhs.len());
        self.inner.iter_mut().zip(rhs.iter()).for_each(f);
        self.reduce();
    }
}

impl<'a, E: WholeNum> std::ops::MulAssign<&'a Urat<E>> for Urat<E> {
    #[inline]
    fn mul_assign(&mut self, rhs: &'a Urat<E>) {
        self.rfr(&rhs.inner[..], |(i, j)| *i = *i + *j);
    }
}

impl<'a, E: WholeNum> std::ops::Mul<&'a Urat<E>> for Urat<E> {
    type Output = Urat<E>;
    #[inline]
    fn mul(mut self, rhs: &'a Urat<E>) -> Urat<E> {
        self *= rhs;
        self
    }
}

impl<'a, E: WholeNum> std::ops::DivAssign<&'a Urat<E>> for Urat<E> {
    #[inline]
    fn div_assign(&mut self, rhs: &'a Urat<E>) {
        self.rfr(&rhs.inner[..], |(i, j)| *i = *i - *j);
    }
}

impl<'a, E: WholeNum> std::ops::Div<&'a Urat<E>> for Urat<E> {
    type Output = Urat<E>;
    #[inline]
    fn div(mut self, rhs: &'a Urat<E>) -> Urat<E> {
        self /= rhs;
        self
    }
}

impl<'a, E: WholeNum> num_traits::Pow<&'a E> for Urat<E> {
    type Output = Urat<E>;
    fn pow(mut self, rhs: &'a E) -> Urat<E> {
        self.inner.iter_mut().for_each(|i| *i = *i * *rhs);
        self.reduce();
        self
    }
}

impl<E: WholeNum> From<std::num::NonZeroU64> for Urat<E> {
    fn from(inp: std::num::NonZeroU64) -> Self {
        let mut inp = inp.get();
        let mut ret = Self::new();
        if inp == 1 {
            // we don't need to acquire the global lock if we don't need any
            // prime numbers anyways
            return ret;
        }
        for (pidx, pval) in crate::primes::iter().enumerate() {
            if inp % pval != 0 {
                continue;
            }
            assert_ne!(inp, 0);
            ret.reserve(pidx + 1);
            let rpirf = &mut ret.inner[pidx];
            while inp % pval == 0 {
                // overflow potential:
                // not even i8 is small enough to not fit (u64 max / 2 + 1) into it.
                *rpirf = *rpirf + E::one();
                // for-ever loop potential:
                // this should never lead to `inp == 0`
                inp /= pval;
                debug_assert_ne!(inp, 0);
            }
            if inp == 1 {
                break;
            }
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use crate::Urat;
    use proptest::prelude::*;

    #[test]
    fn tconv_basic() {
        let inps: [(u64, &[i8]); 6] = [
            (1, &[]),
            (2, &[1]),
            (3, &[0, 1]),
            (4, &[2]),
            (6, &[1, 1]),
            (u64::MAX / 2 + 1, &[63]),
        ];
        for (inp, tst) in inps {
            let r: Urat<i8> = std::num::NonZeroU64::new(inp).unwrap().into();
            assert_eq!(
                r,
                Urat {
                    inner: tst.to_vec(),
                }
            );
        }
    }

    proptest! {
        #[test]
        fn fnzu64_always_reduced(i in 1u64..10_000_000) {
            let mut r: Urat<i8> = std::num::NonZeroU64::new(i).unwrap().into();
            let r2 = r.clone();
            r.reduce();
            assert_eq!(r, r2);
        }
    }
}
