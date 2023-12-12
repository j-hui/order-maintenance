use std::ops::{Not, Shl, Shr};

/// Label (i.e., the "tag") that is used to compare priorities.
///
/// Arithmetic operations are suitably overloaded for labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Label(usize);

impl Label {
    pub(crate) const fn new(n: usize) -> Self {
        Self(n)
    }
    pub(crate) const MAX: Self = Label(usize::MAX);
    pub(crate) const BITS: usize = usize::BITS as usize;
}

impl From<Label> for u128 {
    fn from(l: Label) -> Self {
        l.0 as u128
    }
}

impl PartialEq<usize> for Label {
    fn eq(&self, other: &usize) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<usize> for Label {
    fn partial_cmp(&self, other: &usize) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

macro_rules! impl_label_ops {
    () => {};

    (impl $op:ident<Label> { use $op_impl:ident in $method:ident  } $($toks:tt)*) => {
        impl std::ops::$op<Label> for Label {
            type Output = Self;
            fn $method(self, rhs: Label) -> Self::Output {
                Self(self.0.$op_impl(rhs.0))
            }
        }
        impl_label_ops!{$($toks)*}
    };

    (impl $op:ident<usize> { use $op_impl:ident in $method:ident } $($toks:tt)*) => {
        impl std::ops::$op<usize> for Label {
            type Output = Self;
            fn $method(self, rhs: usize) -> Self::Output {
                Self(self.0.$op_impl(rhs))
            }
        }
        impl_label_ops!{$($toks)*}
    };

    (impl mut $op:ident<Label> { use $op_impl:ident in $method:ident  } $($toks:tt)*) => {
        impl std::ops::$op<Label> for Label {
            fn $method(&mut self, rhs: Label) {
                self.0 = self.0.$op_impl(rhs.0);
            }
        }
        impl_label_ops!{$($toks)*}
    };

    (impl mut $op:ident<usize> { use $op_impl:ident in $method:ident } $($toks:tt)*) => {
        impl std::ops::$op<usize> for Label {
            fn $method(&mut self, rhs: usize) {
                self.0 = self.0.$op_impl(rhs);
            }
        }
        impl_label_ops!{$($toks)*}
    };
}

impl_label_ops! {
    impl Add<Label> { use wrapping_add in add }
    impl Add<usize> { use wrapping_add in add }
    impl Sub<Label> { use wrapping_sub in sub }
    impl Sub<usize> { use wrapping_sub in sub }
    impl Mul<usize> { use wrapping_mul in mul }
    impl Div<usize> { use wrapping_div in div }
    impl Shl<usize> { use shl in shl }
    impl Shr<usize> { use shr in shr }
    impl BitXor<Label> { use bitxor in bitxor }
    impl BitAnd<Label> { use bitand in bitand }

    impl mut AddAssign<Label> { use wrapping_add in add_assign }
    impl mut AddAssign<usize> { use wrapping_add in add_assign }
    impl mut ShlAssign<usize> { use shl in shl_assign }
    impl mut ShrAssign<usize> { use shr in shr_assign }
}

impl Not for Label {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}
