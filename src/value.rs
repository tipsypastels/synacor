use crate::{
    num::u15,
    register::{RegisterIndex, Registers},
};
use anyhow::{Result, anyhow};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Literal(u15),
    RegisterIndex(RegisterIndex),
}

impl Value {
    pub fn new(v: u16) -> Result<Self> {
        if let Ok(literal) = u15::new(v) {
            Ok(Self::Literal(literal))
        } else if let Ok(register) = RegisterIndex::from_exact(v) {
            Ok(Self::RegisterIndex(register))
        } else {
            Err(anyhow!("invalid value or register: '{v}'"))
        }
    }

    pub fn as_literal(self) -> Result<u15> {
        match self {
            Self::Literal(v) => Ok(v),
            Self::RegisterIndex(r) => Err(anyhow!("expected literal, found register {r}")),
        }
    }

    pub fn as_register_index(self) -> Result<RegisterIndex> {
        match self {
            Self::Literal(v) => Err(anyhow!("expected register, found literal {v}")),
            Self::RegisterIndex(r) => Ok(r),
        }
    }

    pub fn as_resolved(self, registers: &Registers) -> u15 {
        match self {
            Self::Literal(v) => v,
            Self::RegisterIndex(r) => registers.get(r),
        }
    }
}
