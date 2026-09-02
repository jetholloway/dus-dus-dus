use ordered_float::OrderedFloat;
use std::iter::Sum;
use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Real {
    value: OrderedFloat<f64>,
}

impl Real {
    pub fn sqrt(self) -> Self {
        self.value.sqrt().into()
    }
}

macro_rules! impl_assign_ops {
    ($self_type: ty, $rhs_type: ty) => {
        impl AddAssign<$rhs_type> for $self_type {
            fn add_assign(&mut self, rhs: $rhs_type) {
                self.value += rhs.value;
            }
        }

        impl SubAssign<$rhs_type> for $self_type {
            fn sub_assign(&mut self, rhs: $rhs_type) {
                self.value -= rhs.value;
            }
        }

        impl MulAssign<$rhs_type> for $self_type {
            fn mul_assign(&mut self, rhs: $rhs_type) {
                self.value *= rhs.value;
            }
        }

        impl DivAssign<$rhs_type> for $self_type {
            fn div_assign(&mut self, rhs: $rhs_type) {
                self.value /= rhs.value;
            }
        }

        impl RemAssign<$rhs_type> for $self_type {
            fn rem_assign(&mut self, rhs: $rhs_type) {
                self.value %= rhs.value;
            }
        }
    };
}

macro_rules! impl_binary_ops {
    ($lhs_type: ty, $rhs_type: ty, $output_type: ty) => {
        impl Add<$rhs_type> for $lhs_type {
            type Output = $output_type;

            fn add(self, rhs: $rhs_type) -> Self::Output {
                let mut result = self.clone();
                result += rhs;
                result
            }
        }

        impl Sub<$rhs_type> for $lhs_type {
            type Output = $output_type;

            fn sub(self, rhs: $rhs_type) -> Self::Output {
                let mut result = self.clone();
                result -= rhs;
                result
            }
        }

        impl Mul<$rhs_type> for $lhs_type {
            type Output = $output_type;

            fn mul(self, rhs: $rhs_type) -> Self::Output {
                let mut result = self.clone();
                result *= rhs;
                result
            }
        }

        impl Div<$rhs_type> for $lhs_type {
            type Output = $output_type;

            fn div(self, rhs: $rhs_type) -> Self::Output {
                let mut result = self.clone();
                result /= rhs;
                result
            }
        }

        impl Rem<$rhs_type> for $lhs_type {
            type Output = $output_type;

            fn rem(self, rhs: $rhs_type) -> Self::Output {
                let mut result = self.clone();
                result %= rhs;
                result
            }
        }
    };
}

impl_assign_ops!(Real, Real);
impl_assign_ops!(Real, &Real);
impl_assign_ops!(Real, &mut Real);

impl_binary_ops!(Real, Real, Real);
impl_binary_ops!(Real, &Real, Real);
impl_binary_ops!(Real, &mut Real, Real);
impl_binary_ops!(&Real, Real, Real);
impl_binary_ops!(&Real, &Real, Real);
impl_binary_ops!(&Real, &mut Real, Real);
impl_binary_ops!(&mut Real, Real, Real);
impl_binary_ops!(&mut Real, &Real, Real);
impl_binary_ops!(&mut Real, &mut Real, Real);

impl Sum for Real {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(0.0.into(), |sum, value| sum + value)
    }
}

impl Neg for Real {
    type Output = Real;

    fn neg(self) -> Self::Output {
        Self { value: -self.value }
    }
}

impl Default for Real {
    fn default() -> Self {
        0.0.into()
    }
}

impl From<f64> for Real {
    fn from(value: f64) -> Self {
        Self {
            value: value.into(),
        }
    }
}
impl From<&f64> for Real {
    fn from(value: &f64) -> Self {
        Self {
            value: OrderedFloat::<f64>::from(*value),
        }
    }
}
impl From<&mut f64> for Real {
    fn from(value: &mut f64) -> Self {
        Self {
            value: OrderedFloat::<f64>::from(*value),
        }
    }
}
