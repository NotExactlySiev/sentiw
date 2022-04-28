#[derive(Debug)]
pub enum Error
{
    IncorrectBufferLength,
    BufferAlreadyLoaded,
    SegmentBufferNotLoaded,
    OffsetOutOfBound,
    SegmentOverlap,
    AddressNotMapped,
    SegmentNotLoaded,
}

#[derive(Debug)]
pub enum Kind
{
    RAM, 
    ROM,
}

#[derive(Debug)]
pub enum Buffer<'a>
{
    NotEnabled,
    ReadOnly(&'a [u8]),
    ReadWrite(&'a mut [u8]),
}

#[derive(Debug)]
pub struct Segment<'a>
{
    start: u16,
    end: u16,
    size: u16, // this is not always implicit since there might be mirroring
    kind: Kind,
    buffer: Buffer<'a>,
    pub ready: bool, // does it have the reference or not
}

    impl<'a> Segment<'a>
    {

        pub fn new(start: u16, end: u16, size: u16, kind: Kind) -> Segment<'a>
        {
            Segment {
                start, end, size, kind,
                buffer: Buffer::NotEnabled,
                ready: false,
            }
        }

        // returns the offset if address is in the segment
        // none otherwise
        pub fn offset(&self, addr: u16) -> Option<u16>
        {
            
            if (addr >= self.start) & (addr <= self.end)
                { Some((addr-self.start)%self.size) }
            else
                { None }
        }

        pub fn overlaps(&self, other: &Segment) -> bool
        {
            if self.start > other.start
                { self.start > other.end }
            else
                { self.end >= other.start }
        }

        pub fn enable_rw(&mut self, slice: &'a mut [u8])
            -> Result<(), Error>
        {
            if self.ready
            {
                return Err(Error::BufferAlreadyLoaded);
            }

            if slice.len() != self.size as usize
            {
                return Err(Error::IncorrectBufferLength);
            }

            //dbg!(slice);
            self.buffer = Buffer::ReadWrite(slice);
            self.ready = true;
            Ok(())
        }

        pub fn enable_ro(&mut self, slice: &'a [u8])
            -> Result<(), Error>
        {
            if self.ready
            {
                return Err(Error::BufferAlreadyLoaded);
            }

            if slice.len() != self.size as usize
            {
                return Err(Error::IncorrectBufferLength);
            }

            //dbg!(slice);
            self.buffer = Buffer::ReadOnly(slice);
            self.ready = true;
            Ok(())
        }

        pub fn disable(&mut self)
            -> Result<(), Error>
        {
            self.buffer = Buffer::NotEnabled;
            self.ready = false;
            Ok(())
        }

        pub fn can_access(&self, offset: u16)
            -> Result<(), Error>
        {
            if !self.ready
            {
                Err(Error::SegmentBufferNotLoaded)
            }
            else if offset >= self.size
            {
                Err(Error::OffsetOutOfBound)
            }
            else
                { Ok(()) }
        }

        pub fn read(&self, offset: u16) -> Result<u8, Error>
        {
            if let Err(e) = self.can_access(offset)
            {
                Err(e)
            }
            else { match &self.buffer
            {
                Buffer::ReadOnly(buf) =>
                    Ok(buf[offset as usize]),
                
                Buffer::ReadWrite(buf) =>
                    Ok(buf[offset as usize]),

                _ => panic!("buffer is not enabled!"),
            }}
        }

        pub fn write(&mut self, offset: u16, data: u8)
            -> Result<(), Error>
        {
            
            if let Err(e) = self.can_access(offset)
            {
                Err(e)
            }
            else
            {
                match &mut self.buffer
                {
                    Buffer::ReadWrite(buf) =>
                    {
                        buf[offset as usize] = data;
                        Ok(())
                    }

                    Buffer::ReadOnly(_) =>
                        panic!("trying to write to a read only segment"),
                    Buffer::NotEnabled =>
                        panic!("the segment is active but no buffer!"),
                }
            }
        }
    }


#[derive(Debug)]
pub struct MemoryMap<'a>
{
    segs: Vec<Segment<'a>>,
}


impl<'a> MemoryMap<'a>
{
    pub fn new() -> MemoryMap<'a>
    {
        MemoryMap {
            segs: Vec::new(),
        }
    }

    pub fn add_seg(&mut self, start: u16, end: u16, size: u16, kind: Kind)
        -> Result<&mut Segment<'a>, Error>
    {
        let new_seg = Segment::new(start, end, size, kind);
        for seg in &self.segs
        {
            if seg.overlaps(&new_seg)
            {
                return Err(Error::SegmentOverlap)
            }
        }

        self.segs.push(new_seg);
        Ok(self.segs.last_mut().unwrap())
    }

    // returns the segment that contains this address,
    // returns None if the address isn't mapped
    pub fn map(&'a self, addr: u16)
        -> Result<(&Segment, u16), Error>
    {
        for seg in &self.segs
        {
            if let Some(offset) = seg.offset(addr)
            {
                return Ok((seg, offset));
            }
        }

        Err(Error::AddressNotMapped)
    }

    // returns a mutable reference to the segment that
    // contains this address
    pub fn map_mut<'b>(&'b mut self, addr: u16)
        -> Result<(&'b mut Segment<'a>, u16), Error>
    {
        for seg in &mut self.segs
        {
            if let Some(offset) = seg.offset(addr)
            {
                return Ok((seg, offset));
            }
        }

        Err(Error::AddressNotMapped)
    }

    pub fn read(&self, addr: u16) -> Result<u8, Error>
    {
        match self.map(addr)
        {
            Ok((seg, offset)) => Ok(seg.read(offset).unwrap()),
            Err(e) => Err(e)
        }
    }

    pub fn write(&mut self, addr: u16, data: u8)
        -> Result<(), Error>
    {
        match self.map_mut(addr)
        {
            Ok((seg, offset)) =>
            {
                seg.write(offset, data);
                Ok(())
            }
            Err(e) => Err(e)
        }
    }

    pub fn enable_seg_rw<'b>(&'b mut self, addr: u16, slice: &'a mut [u8])
        -> Result<(), Error>
    {
        match self.map_mut(addr)
        {
            Ok((seg, _)) => seg.enable_rw(slice),
            Err(e) => Err(e)
        }
    }

    pub fn enable_seg_ro(&mut self, addr: u16, slice: &'a [u8])
        -> Result<(), Error>
    {
        match self.map_mut(addr)
        {
            Ok((seg, _)) => seg.enable_ro(slice),
            Err(e) => Err(e)
        }
    }

    pub fn disable_seg<'b>(&'b mut self, addr: u16)
        -> Result<(), Error>
    {
        match self.map_mut(addr)
        {
            Ok((seg, _)) => seg.disable(),
            Err(e) => Err(e)
        }
    }
}

