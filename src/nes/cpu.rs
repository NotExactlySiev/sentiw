use crate::nes::memory_map::MemoryMap;
use std::fmt::{ Display, Formatter };

const NMI_VECTOR: u16 = 0xfffa;
const RST_VECTOR: u16 = 0xfffc;
const IRQ_VECTOR: u16 = 0xfffe;


const FLAG_N: u8 = 0b1000_0000;
const FLAG_V: u8 = 0b0100_0000;
const FLAG_B: u8 = 0b0001_0000;
const FLAG_D: u8 = 0b0000_1000;
const FLAG_I: u8 = 0b0000_0100;
const FLAG_Z: u8 = 0b0000_0010;
const FLAG_C: u8 = 0b0000_0001;


enum AddrMode
{
    Acc,
    Imp,
    Imm,
}

#[derive(Debug)]
pub struct CPU<'a>
{
    a: u8,
    x: u8,
    y: u8,
    s: u8,
    p: u8,
    pub pc: u16,

    pub mem: MemoryMap<'a>,

    // non state
    cycles: u64, 
}

impl<'a> CPU<'a>
{
    pub fn new() -> CPU<'a>
    {
        CPU {
            a: 0, x: 0, y: 0, p: 0, s: 0xff, pc: 0,
            cycles: 0,
            mem: MemoryMap::new(), 
        }
    }

    pub fn reset(&mut self)
    {
        self.pc = self.read_word(RST_VECTOR);
        self.cycles = 0;
    }

    pub fn write(&mut self, addr: u16, data: u8)
    {
        self.cycles += 1;
        self.mem.write(addr, data)
            .expect("cpu can't write to memory!");
    }

    pub fn read(&mut self, addr: u16) -> u8
    {
        self.cycles += 1;
        self.mem.read(addr)
            .expect("cpu can't read from memory!")
    }

    // maybe this method should exist in the memory module
    // since we might also need it for ppu too
    pub fn read_word(&mut self, addr: u16) -> u16
    {
        self.read(addr) as u16
            | ((self.read(addr+1) as u16) << 8)
    }

    pub fn read_next_byte(&mut self) -> u8
    {
        let result = self.read(self.pc);
        self.pc += 1;
        result
    }

    pub fn read_next_word(&mut self) -> u16
    {
       let result = self.read_word(self.pc);
       self.pc += 2;
       result
    }


    pub fn set_flags(&mut self, f: u8)
    {
        self.p |= f;
    }

    pub fn clear_flags(&mut self, f: u8)
    {
        self.p &= !f;
    }

    // can call with multiple flags or'd together
    pub fn is_flag_set(&self, f: u8) -> bool
    {
        self.p & f != 0
    }

    pub fn step(&mut self)
    {
        let opcode = self.read_next_byte();

        self.cycles += 1;
        // TODO: replace this with an array of fn pointers
        // also operand fetching and execution should be
        // handled seperately so we don't end up writing
        // a closure for every single opcode possible
        match opcode
        {
            0x10 =>
            {
                let operand = self.read_next_byte() as i8;
                if !self.is_flag_set(FLAG_N)
                {
                    (self.pc, _) = self.pc.overflowing_add(operand as u16);
                    self.cycles += 1;
                }

            }

            0x78 =>
                self.set_flags(FLAG_I),

            0x8e =>
            {
                let operand = self.read_next_word();
                self.write(operand, self.x);
            }

            0xa2 =>
            {
                let operand = self.read_next_byte();    
                self.x = operand;
            }

            0xad =>
            {
                let operand = self.read_next_word();
                let operand = self.read(operand);
                self.a = operand;
            }

            0xd8 =>
                self.clear_flags(FLAG_D), 


            other => panic!("opcode {:02x} not implemented", other),
        }
    }

}

impl<'a> Display for CPU<'a>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error>
    {
        write!(f, "A: 0x{:02X} X: 0x{:02X} Y: 0x{:02X}  NV-BDIZC\n\
                  PC: 0x{:04X} S: 0x{:02X}       {:08b}",
                  self.a, self.x, self.y, self.pc, self.s, self.p)
                  
    }
}
