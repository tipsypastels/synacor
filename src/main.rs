mod num;
mod register;
mod value;

use crate::{num::*, register::*, value::*};
use anyhow::{Context, Result};
use hxd::{AsHexdGrouped, options::HexdOptionsBuilder};
use std::{
    fs,
    io::{self, Read},
};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

struct Vm {
    ptr: usize,
    ram: Vec<u16>,
    regs: Registers,
    stack: Vec<u15>,
    halted: bool,
}

impl Vm {
    fn new(rom: Vec<u16>) -> Self {
        Self {
            ptr: 0,
            ram: rom,
            regs: Registers::default(),
            stack: Vec::new(),
            halted: false,
        }
    }

    fn run(&mut self) -> Result<()> {
        while !self.halted {
            self.run_once()
                .with_context(|| format!("error at {:08X}", self.ptr))?;
        }
        println!("halted at {}", self.ptr);
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(ptr = self.ptr))]
    fn run_once(&mut self) -> Result<()> {
        let cmd = self.read_literal_value()?.as_u16();
        match cmd {
            // halt
            0 => {
                tracing::trace!("halt");
                self.halted = true;
            }
            // set a b
            1 => {
                let a = self.read_register_index_value()?;
                let b = self.read_resolved_value()?;
                tracing::trace!("set {a} = {b}");
                self.regs.set(a, b);
            }
            // push a
            2 => {
                let a = self.read_resolved_value()?;
                tracing::trace!("push {a}");
                self.stack.push(a);
            }
            // pop a
            3 => {
                let a = self.read_register_index_value()?;
                tracing::trace!("pop {a}");
                let s = self.stack.pop().context("empty stack")?;
                self.regs.set(a, s);
            }
            // eq a b c
            4 => {
                let a = self.read_register_index_value()?;
                let b = self.read_resolved_value()?;
                let c = self.read_resolved_value()?;
                tracing::trace!("eq {a} {b} {c}");
                self.regs.set(a, if b == c { u15::ONE } else { u15::ZERO });
            }
            // gt a b c
            5 => {
                let a = self.read_register_index_value()?;
                let b = self.read_resolved_value()?;
                let c = self.read_resolved_value()?;
                tracing::trace!("gt {a} {b} {c}");
                self.regs.set(a, if b > c { u15::ONE } else { u15::ZERO });
            }
            // jmp a
            6 => {
                let a = self.read_resolved_value()?;
                tracing::trace!("jmp {a}");
                self.ptr = a.as_usize();
            }
            // jt a b
            7 => {
                let a = self.read_resolved_value()?;
                let b = self.read_resolved_value()?;
                tracing::trace!("jt {a} {b}");
                if a != 0 {
                    self.ptr = b.as_usize();
                }
            }
            // jf a b
            8 => {
                let a = self.read_resolved_value()?;
                let b = self.read_resolved_value()?;
                tracing::trace!("jf {a} {b}");
                if a == 0 {
                    self.ptr = b.as_usize();
                }
            }
            // add a b c
            9 => {
                let a = self.read_register_index_value()?;
                let b = self.read_resolved_value()?;
                let c = self.read_resolved_value()?;
                tracing::trace!("add {a} {b} {c}");
                self.regs.set(a, b + c);
            }
            // mult a b c
            10 => {
                let a = self.read_register_index_value()?;
                let b = self.read_resolved_value()?;
                let c = self.read_resolved_value()?;
                tracing::trace!("mult {a} {b} {c}");
                self.regs.set(a, b * c);
            }
            // mod a b c
            11 => {
                let a = self.read_register_index_value()?;
                let b = self.read_resolved_value()?;
                let c = self.read_resolved_value()?;
                tracing::trace!("mod {a} {b} {c}");
                self.regs.set(a, b % c);
            }
            // and a b c
            12 => {
                let a = self.read_register_index_value()?;
                let b = self.read_resolved_value()?;
                let c = self.read_resolved_value()?;
                tracing::trace!("and {a} {b} {c}");
                self.regs.set(a, b & c);
            }
            // or a b c
            13 => {
                let a = self.read_register_index_value()?;
                let b = self.read_resolved_value()?;
                let c = self.read_resolved_value()?;
                tracing::trace!("or {a} {b} {c}");
                self.regs.set(a, b | c);
            }
            // not a b
            14 => {
                let a = self.read_register_index_value()?;
                let b = self.read_resolved_value()?;
                tracing::trace!("not {a} {b}");
                self.regs.set(a, !b);
            }
            // rmem 15 a b
            15 => {
                let a = self.read_register_index_value()?;
                let b = self.read_resolved_value()?;
                tracing::trace!("rmem {a} {b}");
                self.regs.set(a, u15::new(self.ram[b.as_usize()])?);
            }
            // wmem 16 a b
            16 => {
                let a = self.read_resolved_value()?;
                let b = self.read_resolved_value()?;
                tracing::trace!("wmem {a} {b}");
                self.ram[a.as_usize()] = b.as_u16();
            }
            // call a
            17 => {
                let a = self.read_resolved_value()?;
                tracing::trace!("call {a}");
                self.stack.push(u15::new(self.ptr as _)?);
                self.ptr = a.as_usize();
            }
            // ret
            18 => {
                tracing::trace!("ret");
                if let Some(ptr) = self.stack.pop() {
                    self.ptr = ptr.as_usize();
                } else {
                    self.halted = true;
                }
            }
            // out a
            19 => {
                let a = self.read_resolved_value()?.as_char()?;
                print!("{a}");
            }
            // in a
            20 => {
                let mut buf = [0];

                macro_rules! read_byte {
                    () => {
                        io::stdin()
                            .read_exact(&mut buf)
                            .context("failed to read stdin")?;
                    };
                }

                read_byte!();

                while buf[0] == b'?' {
                    self.read_internal_command()?;
                    read_byte!();
                }

                let a = self.read_register_index_value()?;
                tracing::trace!("in {a}");
                self.regs.set(a, u15::new(buf[0] as _)?);
            }
            // noop
            21 => {
                tracing::trace!("noop");
            }
            _ => {
                unreachable!("unknown instruction {} at {}", cmd, self.ptr);
            }
        }
        Ok(())
    }

    fn read_resolved_value(&mut self) -> Result<u15> {
        self.read_value().map(|v| v.as_resolved(&self.regs))
    }

    fn read_literal_value(&mut self) -> Result<u15> {
        self.read_value().and_then(|v| v.as_literal())
    }

    fn read_register_index_value(&mut self) -> Result<RegisterIndex> {
        self.read_value().and_then(|v| v.as_register_index())
    }

    fn read_value(&mut self) -> Result<Value> {
        let v = Value::new(self.ram[self.ptr])?;
        self.ptr += 1;
        Ok(v)
    }

    fn read_internal_command(&mut self) -> Result<()> {
        let mut buf = String::new();

        io::stdin()
            .read_line(&mut buf)
            .context("failed to read internal command")?;

        match buf.trim() {
            "d" | "dump" => {
                let file = fs::OpenOptions::new()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .open("dump.hex")
                    .context("failed to open dump out file")?;

                self.ram
                    .hexd_le()
                    .dump_io(file)
                    .context("hex dump failed")?;

                println!("Hex dumped.");
            }
            "r" | "regs" => {
                self.regs.hexd_le().show_index(false).dump();
            }
            _ => {
                unreachable!("unknown internal command '{buf}'");
            }
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false).without_time())
        .with(EnvFilter::from_default_env())
        .init();

    let rom = fs::read("challenge.bin")
        .context("failed to read binary")?
        .chunks_exact(2)
        .map(|a| u16::from_le_bytes([a[0], a[1]]))
        .collect::<Vec<u16>>();

    let mut vm = Vm::new(rom);
    vm.run()
}
