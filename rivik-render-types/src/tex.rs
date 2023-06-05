use std::mem;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Tex<const L: usize> {
    width: u16,
    height: u16,
    data: [u8; L],
}

#[derive(Clone, Debug)]
pub struct DynTex {
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
}

pub trait AsTex {
    fn texel_width(&self) -> u16 {
        (self.buffer().len() / self.height() as usize / self.width() as usize) as u16
    }
    fn width(&self) -> u16;
    fn height(&self) -> u16;
    fn buffer(&self) -> &[u8];
}

impl<const L: usize> AsTex for Tex<L> {
    fn buffer(&self) -> &[u8] {
        &self.data
    }

    fn height(&self) -> u16 {
        self.height
    }

    fn width(&self) -> u16 {
        self.width
    }
}

impl AsTex for DynTex {
    fn height(&self) -> u16 {
        self.height
    }

    fn buffer(&self) -> &[u8] {
        &self.data
    }

    fn width(&self) -> u16 {
        self.width
    }
}
