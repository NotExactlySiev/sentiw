
// registers
const PPU_CRTL: usize = 0;
const PPU_MASK: usize = 1;
const PPU_STATUS: usize = 2;
const OAM_ADDR: usize = 3;
const OAM_DATA: usize = 4;
const PPU_SCROLL: usize = 5;
const PPU_ADDR: usize = 6;
const PPU_DATA: usize = 7;

#[derive(Debug)]
enum Version
{
    NTSC,
    PAL,
}

#[derive(Debug)]
pub struct PPU
{
    regs: [u8; 8],
    version: Version
}


impl PPU
{
    pub fn new() -> PPU
    {
        PPU {
            regs: [0u8; 8],
            version: Version::NTSC
        }
    }

    // puts the ppu in the start of power up state
    // TODO: there should be two functions implemented,
    // one for reset and one for power up
    pub fn reset(&mut self)
    {
        match self.version
        {
            Version::NTSC =>
            {
                todo!();
            }


            _ => panic!("This version of the PPU is not implemented"),
        }
    }

    pub fn regs_slice(&mut self) -> &mut [u8]
    {
        &mut self.regs
    }
}
