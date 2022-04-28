pub mod nes;

use nes::NES;
use nes::ines::{ INesRom, Error };
use std::fs::{ File };

fn main()
{

    let mut romfile = File::open("tetris.nes")
        .expect("failed to open rom file");

    let ines = match INesRom::new(&mut romfile)
    {
        Ok(rom) => rom,
        Err(e) => match e
        {
            Error::MapperNotSupported => panic!("This mapper is not supported yet"),
            Error::HeaderNotFound => panic!("Header data was not found"),
            other => panic!("{:?}", other), 
        }
    };

    println!("Rom loaded from file");
    println!("{:?}\n", ines);

    let mut nes = NES::new();
    //nes.pre_setup();
    nes.load_cart(ines);
    nes.run();

}
