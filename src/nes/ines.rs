// This module reads and parses data from the ines files
// it doesn't do mapping it doesn't do anything else!
// only reads and parses .nes files

use std::fs::{ File };
use std::io::{ Read };
use std::cmp::{ Ordering };
use std::result::Result::{ self, Ok, Err };
use std::fmt::{ Debug, Formatter };

const MAGIC_NUM: &[u8] = &[78, 69, 83, 26];
const PRG_BANK_SIZE: usize = 0x4000;
const CHR_BANK_SIZE: usize = 0x2000;



#[derive(Debug)]
pub enum Error
{
    ReadError(std::io::Error),
    HeaderNotFound,
    PrgSizeMismatch,
    ChrSizeMismatch,
    MapperNotSupported,
}

#[derive(Debug, PartialEq)]
pub enum HeaderKind
{
    NoHeader,
    OldINES, // Archaic iNes
    INES,    // iNes
    INES2,   // iNes2.0
}

#[derive(Debug)]
pub enum Mirroring
{
    Horizontal,
    Vertical,
    FourScreen
}

pub struct INesRom
{
    header_kind: HeaderKind,
    pub buffer: Vec<u8>,
    pub size: usize,
    pub prg_offset: usize,
    pub chr_offset: usize,

    pub is_loaded: bool,
    pub prg_count: u8,
    pub chr_count: u8,
    pub mapper: u16,
    pub mirroring: Mirroring,
    pub has_nvram: bool,
    pub has_trainer: bool,
}



impl INesRom
{

    fn new_empty() -> INesRom 
    {
        INesRom {
            buffer: Vec::new(),
            size: 0,
            prg_offset: 0,
            chr_offset: 0,
            is_loaded: false,
            prg_count: 0,
            chr_count: 0,
            mapper: 0,
            mirroring: Mirroring::Horizontal,
            has_nvram: false,
            has_trainer: false,
            header_kind: HeaderKind::INES,
        }
    }

    pub fn header(&self) -> &[u8]
    {
        &self.buffer[0..16]
    }

    pub fn new(file: &mut File) -> Result<Self, Error>
    {
        let mut out = Self::new_empty();
        match file.read_to_end(&mut out.buffer)
        {
            Ok(size) => out.size = size,
            Err(error) => return Err(Error::ReadError(error))
        }
        // should it call another function like
        // parse_header or something after this?

        if (&out.header()[0..4]).cmp(MAGIC_NUM) != Ordering::Equal
        {
            return Err(Error::HeaderNotFound);
        }
        
        out.header_kind = HeaderKind::INES;
        // TODO: detect header type
        // accodrding to nesdev:
        // If byte 7 AND $0C = $08, and the size taking into account 
        // byte 9 does not exceed the actual size of the ROM image,
        //then NES 2.0.
        // If byte 7 AND $0C = $00, and bytes 12-15 are all 0, then iNES.
        // Otherwise, archaic iNES.




        // Header is present. Read metadata
        out.mapper = (out.buffer[6] >> 4 | out.buffer[7] & 0xf0) as u16;
        
        // TODO: some of the following steps will be different for different mappers.
        // use match for adding support in the future
        
        match out.mapper {
            0..=1 => 
            {
                out.mirroring = if out.buffer[6] & 8 == 0
                {
                  if out.buffer[6] & 1 == 0
                  { Mirroring::Horizontal }
                  else
                  { Mirroring::Vertical }
                }
                else { Mirroring::FourScreen };
            },
            _ => return Err(Error::MapperNotSupported),
        }


        out.prg_count = out.buffer[4];
        out.chr_count = out.buffer[5];
        out.has_trainer = out.buffer[6] & 4 != 0;

        out.prg_offset = 0
            + if out.header_kind != HeaderKind::NoHeader { 16 } else { 0 }
            + if out.has_trainer { 512 } else { 0 };
                
        out.chr_offset = out.prg_offset + out.prg_size();

        out.is_loaded = true;
        

        Ok(out)
    }

    pub fn prg_size(&self) -> usize
    {
        PRG_BANK_SIZE * self.prg_count as usize
    }

    pub fn chr_size(&self) -> usize
    {
        CHR_BANK_SIZE * self.chr_count as usize
    }

    // these should really be handled by the console maybe
    pub fn prg_bank(&self, index: u8) -> &[u8]
    {
        let index = index as usize;
        let offset = self.prg_offset;
        let begin = offset + PRG_BANK_SIZE*index;
        let end = begin + PRG_BANK_SIZE;
        &self.buffer[begin..end]
    }

    pub fn prg_bank_mut(&mut self, index: u8) -> &mut [u8]
    {
        let index = index as usize;
        let offset = self.prg_offset;
        let begin = offset + PRG_BANK_SIZE*index;
        let end = begin + PRG_BANK_SIZE;
        &mut self.buffer[begin..end]
    }
}


impl Debug for INesRom
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error>
    {
        write!(f, "Mapper: {}\nPRG: {}K\nCHR: {}K\nMirroring: {:?}",
               self.mapper,
               self.prg_size()/1024,
               self.chr_size()/1024, 
               self.mirroring).unwrap();
        Ok(())
    }
}
