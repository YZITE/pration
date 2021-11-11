pub mod primes;
use crate::primes::PRIMES;

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
        Self {
            inner: vec![],
        }
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
        while self.inner.last() == Some(&zero) {
            self.inner.pop();
        }
    }

    fn rfr<F: FnMut((&mut E, &E),)>(&mut self, rhs: &[E], f: F) {
        self.reserve(rhs.len());
        self.inner.iter_mut().zip(rhs.into_iter()).for_each(f);
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

#[derive(Clone, Debug, thiserror::Error)]
#[error("some output value exponent overflowed")]
pub struct OverflowError;

impl<E: WholeNum> TryFrom<std::num::NonZeroU64> for Urat<E> {
    type Error = OverflowError;
    fn try_from(inp: std::num::NonZeroU64) -> Result<Self, OverflowError> {
        let mut inp = inp.get();
        let mut ret = Self::new();
        let one = E::one();
        for (pidx, pval) in PRIMES.write().expect("accessing PRIMES failed").iter().enumerate() {
            if inp == 1 {
                break;
            } else if inp % pval == 0 {
                ret.reserve(pidx + 1);
                let rpirf = &mut ret.inner[pidx];
                while inp % pval == 0 {
                    assert!(inp > 0);
                    if let Some(x) = rpirf.checked_add(&one) {
                        *rpirf = x;
                    } else {
                        return Err(OverflowError);
                    }
                    inp /= pval;
                }
            }
        }
        // shouldn't be necessary, but I'm not sure if the code above has no leaks
        ret.reduce();
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use crate::Urat;
    #[test]
    fn tconv_basic() {
        use std::num::NonZeroU64;
        let o = NonZeroU64::new(1).unwrap();
        let w = NonZeroU64::new(2).unwrap();
        let e = NonZeroU64::new(3).unwrap();
        let ro: Urat<i8> = o.try_into().unwrap();
        let rw: Urat<i8> = w.try_into().unwrap();
        let re: Urat<i8> = e.try_into().unwrap();
        assert_eq!(ro, Urat::default());
        assert_eq!(rw, Urat {
            inner: vec![1],
        });
        assert_eq!(re, Urat {
            inner: vec![0, 1],
        });
    }
}
