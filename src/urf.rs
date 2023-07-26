use std::{mem::size_of, slice::from_raw_parts};

/// Data types used in the URF header
mod types {
    #[allow(dead_code)]
    #[repr(u8)]
    pub enum ColorSpace {
        Sgray = 0_u8.to_be(),
        Srgb = 1_u8.to_be(),
        CieLab = 2_u8.to_be(),
        AdobeRgb = 3_u8.to_be(),
        Gray32 = 4_u8.to_be(),
        RGB = 5_u8.to_be(),
        CMYK = 6_u8.to_be(),
    }

    #[allow(dead_code)]
    #[repr(u8)]
    pub enum Duplex {
        NoDuplex = 1_u8.to_be(),
        ShortSide = 2_u8.to_be(),
        LongSide = 3_u8.to_be(),
    }

    #[allow(dead_code)]
    #[repr(u8)]
    pub enum MediaPosition {
        Auto = 0_u8.to_be(),
        Main = 1_u8.to_be(),
        Alternate = 2_u8.to_be(),
        LargeCapacity = 3_u8.to_be(),
        Manual = 4_u8.to_be(),
        Envelope = 5_u8.to_be(),
        Disc = 6_u8.to_be(),
        Photo = 7_u8.to_be(),
        Hagaki = 8_u8.to_be(),
        MainRoll = 9_u8.to_be(),
        AlternateRoll = 10_u8.to_be(),
        Top = 11_u8.to_be(),
        Middle = 12_u8.to_be(),
        Bottom = 13_u8.to_be(),
        Side = 14_u8.to_be(),
        Left = 15_u8.to_be(),
        Right = 16_u8.to_be(),
        Center = 17_u8.to_be(),
        Rear = 18_u8.to_be(),
        ByPassTray = 19_u8.to_be(),
        Tray1 = 20_u8.to_be(),
        Tray2 = 21_u8.to_be(),
        Tray3 = 22_u8.to_be(),
        Tray4 = 23_u8.to_be(),
        Tray5 = 24_u8.to_be(),
        Tray6 = 25_u8.to_be(),
        Tray7 = 26_u8.to_be(),
        Tray8 = 27_u8.to_be(),
        Tray9 = 28_u8.to_be(),
        Tray10 = 29_u8.to_be(),
        Tray11 = 30_u8.to_be(),
        Tray12 = 31_u8.to_be(),
        Tray13 = 32_u8.to_be(),
        Tray14 = 33_u8.to_be(),
        Tray15 = 34_u8.to_be(),
        Tray16 = 35_u8.to_be(),
        Tray17 = 36_u8.to_be(),
        Tray18 = 37_u8.to_be(),
        Tray19 = 38_u8.to_be(),
        Tray20 = 39_u8.to_be(),
        Roll1 = 40_u8.to_be(),
        Roll2 = 41_u8.to_be(),
        Roll3 = 42_u8.to_be(),
        Roll4 = 43_u8.to_be(),
        Roll5 = 44_u8.to_be(),
        Roll6 = 45_u8.to_be(),
        Roll7 = 46_u8.to_be(),
        Roll8 = 47_u8.to_be(),
        Roll9 = 48_u8.to_be(),
        Roll10 = 49_u8.to_be(),
    }

    #[allow(dead_code)]
    #[repr(u8)]
    pub enum MediaType {
        AutomaticMediaType = 0_u8.to_be(),
        Stationery = 1_u8.to_be(),
        Transparency = 2_u8.to_be(),
        Envelope = 3_u8.to_be(),
        Cardstock = 4_u8.to_be(),
        Labels = 5_u8.to_be(),
        StationeryLetterhead = 6_u8.to_be(),
        Disc = 7_u8.to_be(),
        PhotographicMatte = 8_u8.to_be(),
        PhotographicSatin = 9_u8.to_be(),
        PhotographicSemiGloss = 10_u8.to_be(),
        PhotographicGlossy = 11_u8.to_be(),
        PhotographicHighGloss = 12_u8.to_be(),
        OtherMediaType,
    }

    #[allow(dead_code)]
    #[repr(u8)]
    pub enum Quality {
        Default = 0_u8.to_be(),
        Draft = 3_u8.to_be(),
        Normal = 4_u8.to_be(),
        High = 5_u8.to_be(),
    }

    pub struct Reserved<const N: usize>([u8; N]);

    impl<const N: usize> Default for Reserved<N> {
        fn default() -> Self {
            Self([0; N])
        }
    }
}

use types::*;

use crate::render::A4Pixels;

const URF_SYNC_WORD: &[u8] = b"UNIRAST\0";

#[repr(C, packed)]
#[allow(non_snake_case)]
struct PageHeader {
    BitsPerPixel: u8,
    ColorSpace: ColorSpace,
    Duplex: Duplex,
    Quality: Quality,
    MediaType: MediaType,
    MediaPosition: MediaPosition,
    Reserved1: Reserved<6>,
    Width: u32,
    Height: u32,
    HWRes: u32,
    Reserved2: Reserved<8>,
}

impl PageHeader {
    pub fn new(page_pixels: &A4Pixels) -> Self {
        Self {
            BitsPerPixel: (page_pixels.bits_per_pixel() as u8).to_be(),
            ColorSpace: ColorSpace::Sgray,
            Duplex: Duplex::NoDuplex,                 // TODO?
            Quality: Quality::Default,                // TODO
            MediaType: MediaType::AutomaticMediaType, // TODO
            MediaPosition: MediaPosition::Auto,       // TODO
            Reserved1: Default::default(),
            Width: (page_pixels.width() as u32).to_be(),
            Height: (page_pixels.height() as u32).to_be(),
            HWRes: (page_pixels.resolution_width() as u32).to_be(),
            Reserved2: Default::default(),
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { from_raw_parts((self as *const Self) as *const u8, size_of::<Self>()) }
    }
}

pub(crate) fn create_page(pixels: &A4Pixels, pages: u32, compressed: &[u8]) -> Vec<u8> {
    let len = URF_SYNC_WORD.len() + size_of::<PageHeader>() + compressed.len();
    let mut page = Vec::with_capacity(len);
    page.extend_from_slice(URF_SYNC_WORD);
    page.extend_from_slice(pages.to_be_bytes().as_slice());
    page.extend_from_slice(PageHeader::new(pixels).as_slice());
    page.extend_from_slice(&compressed);
    page
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::PageHeader;

    pub const URF_HEADER_SIZE: usize = 32;

    #[test]
    fn test_page_size_matches() {
        assert_eq!(URF_HEADER_SIZE, size_of::<PageHeader>())
    }
}
