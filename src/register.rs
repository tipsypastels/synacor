use crate::num::u15;
use anyhow::{Result, anyhow};
use hxd::{AsHexdGrouped, Hexd, options::Endianness, reader::GroupedSliceByteReader};
use std::fmt;

#[derive(Default, Debug)]
pub struct Registers([u15; RegisterIndex::COUNT]);

impl Registers {
    pub fn get(&self, register: RegisterIndex) -> u15 {
        self.0[register.0 as usize]
    }

    pub fn set(&mut self, register: RegisterIndex, value: u15) {
        self.0[register.0 as usize] = value;
    }
}

impl<'a> AsHexdGrouped<'a, GroupedSliceByteReader<'a, u15, 2>> for Registers {
    fn as_hexd_grouped(&'a self, end: Endianness) -> Hexd<GroupedSliceByteReader<'a, u15, 2>> {
        self.0.as_hexd_grouped(end)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegisterIndex(u8);

impl RegisterIndex {
    pub const COUNT: usize = 8;

    pub fn new(v: u16) -> Result<Self> {
        if v as usize >= Self::COUNT {
            return Err(anyhow!("invalid register: '{v}'"));
        }
        Ok(Self(v as _))
    }

    pub fn from_exact(v: u16) -> Result<Self> {
        Self::new(v % (2u16.pow(15)))
    }
}

impl fmt::Display for RegisterIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
