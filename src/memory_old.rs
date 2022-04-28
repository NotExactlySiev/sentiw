// This module manages the console bus and all memory access

use super::ines::INesRom;

use std::result::Result::{ self, Ok, Err };

// TODO: maybe put all the constants in one module?
const ADDR_SIZE: usize = 16;

pub enum MemoryAccess<'a>
{
    Ram(&'a mut [u8]), // Internal and WRAM, R/W
    Rom(&'a [u8]), // Cartridge ROM, R
    PPUReg(&'a mut [u8]),
    APUReg,
    IOReg,
    CartReg,
    Void,          // Non-exsistent memory, garbage
}

#[derive(Debug)]
pub enum Error
{
    RomNotLoaded
}

// this struct acts as the main hub, and has (mutable) references
// to all memory on the console
pub struct Memory<'a>
{
    // cpu memory map
    ram: [u8; 0x800],
    wram: [u8; 0x2000],
    prg_lo: Option<&'a [u8]>,
    prg_hi: Option<&'a [u8]>,

    // physical memory
    rom: Option<&'a INesRom>,
    // mutable slice to ppu memory
    ppu_regs: Option<&'a mut [u8]>

}

impl<'a> Memory<'a>
{
    
    pub fn new(ppu_regs: Option<&'a mut [u8]>) -> Memory<'a>
    {
        Memory {
            ram: [0xff; 0x800],
            wram: [0xff; 0x2000],
            prg_lo: None,
            prg_hi: None,
            rom: None,
            ppu_regs,
        }
    }

    pub fn load_rom(mut self, rom: &'a INesRom) -> Result<Self, Error>
    {
        self.rom = Some(rom);
        self.setup();
        Ok(self)
    }

    pub fn setup(&mut self) -> Result<(), Error>
    {
        match self.rom
        {
            None => return Err(Error::RomNotLoaded),
            Some(rom) => {
                match rom.mapper
                {
                    0..=1 => {
                        let last = rom.prg_count - 1;
                        self.prg_lo = Some(rom.prg_bank(0));
                        self.prg_hi = Some(rom.prg_bank(last));
                    },

                    _ => todo!(),
                    
                
                }
            }
        }

        Ok(())
    }

    // this method takes in an address and returns a slice
    // and the address in that slice that corresponds to
    // the given address
    // maybe should be pub

    // oh shit this should return an enum that says what
    // operation should be done when writing to or reading from
    // the given address. if it is even writable or if it's a
    // ppu register etc.

    // returns (memory segment, mirrored addr/reg index)
    // maybe it's not a very well designed method idk
    // but writing two or more identical matches across several
    // functions seems pointless. I put all of the things we need
    // a match for into this one so it's more efficient

    // later note: this method forces all memory access methods
    // that use it (basically all of them) to have a mut
    // reference to self, even ones that only read from
    // the returned slice
    pub fn map(&mut self, addr: u16) -> (MemoryAccess, u16)
    {
        match addr
        {
            0x0000..=0x1FFF => // RAM
                (MemoryAccess::Ram(&mut self.ram), addr%0x400),

            0x2000..=0x3FFF => // PPU Registers
                match self.ppu_regs
                {
                    Some(regs) =>
                        (MemoryAccess::PPUReg(regs), addr%8), 

                    None =>
                        panic!("cpu doesn't have access to ppu registers at this moment"),
                }


            0x4014 =>          // OAM DMA
                panic!("oam dma is not implemented yet!"),

            0x4016 | 0x4017 => // IO Registers
                (MemoryAccess::IOReg, addr%2),

            0x4000..=0x4017 => // APU Registers
                (MemoryAccess::APUReg, addr%24),
                
            0x4018..=0x401F => // Test    
                panic!("CPU test features not implemented yet"),

            0x4020..=0x5FFF => // What?
                panic!("I have no idea what this area of memory does"),

            0x6000..=0x7FFF => // Work RAM
                (MemoryAccess::Ram(&mut self.wram), addr-0x6000),
                
            0x8000..=0xBFFF => // Cartridge ROM
                match self.prg_lo
                {
                    Some(prg) => (MemoryAccess::Rom(prg), addr-0x8000),
                    None => panic!("Cartridge hasn't been loaded into mem yet!")
                },

            0xC000..=0xFFFF => // Cartridge ROM
                match self.prg_hi
                {
                    Some(prg) => (MemoryAccess::Rom(prg), addr-0xC000),
                    None => panic!("Cartridge hasn't been loaded into mem yet!")
                },
        }
    }

    pub fn read(&mut self, addr: u16) -> u8
    {
        let (sl, addr) = self.map(addr);
        match sl
        {
            MemoryAccess::Ram(sl) => sl[addr as usize],
            MemoryAccess::Rom(sl) => sl[addr as usize],
            MemoryAccess::PPUReg(sl) => sl[addr as usize],
            _ => panic!("this read is not handled"),
        }
    }

    pub fn write(&mut self, addr: u16, data: u8)
    {
        let (sl, addr) = self.map(addr);
        match sl
        {
            MemoryAccess::Ram(sl) =>
                sl[addr as usize] = data,

            MemoryAccess::Rom(sl) =>
                println!("write to rom. nothing happened"),

            MemoryAccess::PPUReg(sl) =>
                sl[addr as usize] = data,

            _ => panic!("this write is not handled"),
        }
    }

    pub fn read_word(&mut self, addr: u16) -> u16
    {
        self.read(addr) as u16 
            + ((self.read(addr+1) as u16) << 8)
    }
}
