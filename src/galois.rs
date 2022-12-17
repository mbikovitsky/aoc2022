use std::{
    fmt::Debug,
    num::NonZeroU32,
    ops::{Add, AddAssign, Mul, MulAssign},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GF {
    order: Option<NonZeroU32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GFInt {
    field: GF,
    value: u32,
}

impl GF {
    pub fn new(order: Option<NonZeroU32>) -> Self {
        Self { order }
    }

    pub fn order(&self) -> Option<NonZeroU32> {
        self.order
    }

    pub fn create_value(&self, value: u32) -> GFInt {
        GFInt {
            field: *self,
            value: match self.order {
                Some(order) => value % order,
                None => value,
            },
        }
    }
}

impl GFInt {
    pub fn field(&self) -> GF {
        self.field
    }

    pub fn value(&self) -> u32 {
        self.value
    }

    pub fn square(&self) -> Self {
        (*self) * (*self)
    }

    pub fn square_assign(&mut self) {
        *self = self.square();
    }
}

impl Add for GFInt {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(self.field, rhs.field);

        Self {
            field: self.field,
            value: match self.field.order {
                Some(order) => self.value.checked_add(rhs.value).unwrap() % order,
                None => self.value.wrapping_add(rhs.value),
            },
        }
    }
}

impl AddAssign for GFInt {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Mul for GFInt {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        assert_eq!(self.field, rhs.field);

        Self {
            field: self.field,
            value: match self.field.order {
                Some(order) => self.value.checked_mul(rhs.value).unwrap() % order,
                None => self.value.wrapping_mul(rhs.value),
            },
        }
    }
}

impl MulAssign for GFInt {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use super::GF;

    #[test]
    fn addition() {
        let field = GF::new(NonZeroU32::new(11));
        assert_eq!((field.create_value(6) + field.create_value(5)).value(), 0);
    }

    #[test]
    fn multiplication() {
        let field = GF::new(NonZeroU32::new(11));
        assert_eq!((field.create_value(6) * field.create_value(5)).value(), 8);
    }
}
