pub mod cpu;
pub mod ppu;
pub mod memory_map;
pub mod ines;

use ppu::PPU;
use cpu::CPU;
use ines::{ INesRom, Error };
use memory_map::{ Kind, MemoryMap };

#[derive(Debug)]
pub struct NES<'a>
{
    cpu: CPU<'a>,
    ppu: PPU,
    ram: [u8; 0x800], 
    vram: [u8; 0x1000],
    cart: Option<INesRom>,
    // cart: Cart
    // apu
    // controller
}

impl<'a> NES<'a>
{
    pub fn new() -> NES<'a>
    {
        let mut cpu = CPU::new();
        let mut ppu = PPU::new();

        // Setting up the memory for all the devices
        cpu.mem.add_seg(0x0000, 0x1FFF, 0x800,  Kind::RAM);
        cpu.mem.add_seg(0x8000, 0xBFFF, 0x4000, Kind::ROM);
        cpu.mem.add_seg(0xC000, 0xFFFF, 0x4000, Kind::ROM);


        NES {
            cpu, ppu,
            cart: None,
            ram: [0; 0x800],
            vram: [0; 0x1000],
        }
    }

    pub fn run(&'a mut self)
    {
        self.cpu.mem.enable_seg_rw(0x0000, &mut self.ram);
        self.cpu.mem.enable_seg_rw(0x8000, &mut self.vram);
        // ok i got it. i'm borrowing mem immutebly so i can give back a
        // mutable range from it. i shouldn't do that maybe? or at least
        // define the life times in better terms. right now I can only
        // map_mut once because it borrows mem and keeps it for ever
        
        //let (seg, _) = self.cpu.mem.map_mut(0x8000).unwrap();

        self.cpu.reset();
        loop
        {
            println!("{}", &self.cpu);
            self.cpu.step();
        }
    }


    pub fn reset(&mut self)
    {
        todo!();
    }

    // do we need one more layer of abstraction here?
    // inesrom -> cart -> nes instead of directly
    pub fn load_cart<'b>(&'b mut self, cart: INesRom)
    {
        self.cart = Some(cart);
        if let Some(cart) = &self.cart
        {
            let sl = &cart.buffer[0..10];
            match cart.mapper
            {
                0..=1 =>
                    self.cpu.mem.enable_seg_ro(0x8000, sl),

                _ => todo!()
            };
        };
    }

}

