use anyhow::{Context, Result, anyhow};
use std::{fmt, ops};

#[allow(non_camel_case_types)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct u15(u16);

impl u15 {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);
    pub const MAX: Self = Self(32767);

    pub fn new(v: u16) -> Result<Self> {
        if v > Self::MAX.0 {
            return Err(anyhow!("invalid u15: '{v}'"));
        }
        Ok(Self(v))
    }

    pub const fn as_u16(self) -> u16 {
        self.0
    }

    pub const fn as_usize(self) -> usize {
        self.0 as _
    }

    pub fn as_char(self) -> Result<char> {
        char::from_u32(self.0 as _).with_context(|| format!("invalid char: '{self}'"))
    }
}

impl PartialEq<u16> for u15 {
    fn eq(&self, other: &u16) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<u16> for u15 {
    fn partial_cmp(&self, other: &u16) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl ops::Add for u15 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(((self.0 as u64 + rhs.0 as u64) % 2u64.pow(15)) as _)
    }
}

impl ops::Mul for u15 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(((self.0 as u64 * rhs.0 as u64) % 2u64.pow(15)) as _)
    }
}

impl ops::Rem for u15 {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}

impl ops::BitAnd for u15 {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self((self.0 & rhs.0) % (2u16.pow(15)))
    }
}

impl ops::BitOr for u15 {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self((self.0 | rhs.0) % (2u16.pow(15)))
    }
}

impl ops::Not for u15 {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self((!self.0) % (2u16.pow(15)))
    }
}

impl fmt::Display for u15 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
