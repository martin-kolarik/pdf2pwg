use std::{io::Write, mem::size_of, slice::from_raw_parts};

mod types {
    // 4.3.1.1
    pub struct Boolean(u32);

    impl Boolean {
        pub fn new(value: impl Into<bool>) -> Self {
            Self((value.into() as u32).to_be())
        }
    }

    // 4.3.1.2
    #[repr(C, packed)]
    pub struct CString([u8; 64]);

    impl CString {
        pub fn new(string: &str) -> Self {
            let mut bytes = [0u8; 64];
            let string = string.as_bytes();
            let len = string.len().min(64);
            if len > 0 {
                string
                    .iter()
                    .enumerate()
                    .for_each(|(index, &ch)| bytes[index] = ch);
                bytes[len] = 0;
            }
            Self(bytes)
        }
    }

    impl Default for CString {
        fn default() -> Self {
            Self::new("")
        }
    }

    #[repr(u32)]
    pub enum ColorOrder {
        // Chunky pixels, e.g. CMYK CMYK CMYK ...
        Chunky = 0_u32.to_be(),
    }

    // 4.3.1.4
    #[allow(dead_code)]
    #[repr(u32)]
    pub enum ColorSpace {
        Rgb = 1_u32.to_be(),
        Black = 3_u32.to_be(),
        Cmyk = 6_u32.to_be(),
        Sgray = 18_u32.to_be(),
        Srgb = 19_u32.to_be(),
        AdobeRgb = 20_u32.to_be(),
        Device1 = 48_u32.to_be(),
        Device2 = 49_u32.to_be(),
        Device3 = 50_u32.to_be(),
        Device4 = 51_u32.to_be(),
        Device5 = 52_u32.to_be(),
        Device6 = 53_u32.to_be(),
        Device7 = 54_u32.to_be(),
        Device8 = 55_u32.to_be(),
        Device9 = 56_u32.to_be(),
        Device10 = 57_u32.to_be(),
        Device11 = 58_u32.to_be(),
        Device12 = 59_u32.to_be(),
        Device13 = 60_u32.to_be(),
        Device14 = 61_u32.to_be(),
        Device15 = 62_u32.to_be(),
    }

    // 4.3.1.5
    #[allow(dead_code)]
    #[repr(u32)]
    pub enum Edge {
        ShortEdgeFirst = 0_u32.to_be(),
        LongEdgeFirst = 1_u32.to_be(),
    }

    // 4.3.1.6
    pub struct Integer(i32);

    impl Integer {
        pub fn new(value: impl Into<i32>) -> Self {
            Self(value.into().to_be())
        }
    }

    impl Default for Integer {
        fn default() -> Self {
            Self(0)
        }
    }

    // 4.3.1.7
    #[allow(dead_code)]
    #[repr(u32)]
    pub enum MediaPosition {
        Auto = 0_u32.to_be(),
        Main = 1_u32.to_be(),
        Alternate = 2_u32.to_be(),
        LargeCapacity = 3_u32.to_be(),
        Manual = 4_u32.to_be(),
        Envelope = 5_u32.to_be(),
        Disc = 6_u32.to_be(),
        Photo = 7_u32.to_be(),
        Hagaki = 8_u32.to_be(),
        MainRoll = 9_u32.to_be(),
        AlternateRoll = 10_u32.to_be(),
        Top = 11_u32.to_be(),
        Middle = 12_u32.to_be(),
        Bottom = 13_u32.to_be(),
        Side = 14_u32.to_be(),
        Left = 15_u32.to_be(),
        Right = 16_u32.to_be(),
        Center = 17_u32.to_be(),
        Rear = 18_u32.to_be(),
        ByPassTray = 19_u32.to_be(),
        Tray1 = 20_u32.to_be(),
        Tray2 = 21_u32.to_be(),
        Tray3 = 22_u32.to_be(),
        Tray4 = 23_u32.to_be(),
        Tray5 = 24_u32.to_be(),
        Tray6 = 25_u32.to_be(),
        Tray7 = 26_u32.to_be(),
        Tray8 = 27_u32.to_be(),
        Tray9 = 28_u32.to_be(),
        Tray10 = 29_u32.to_be(),
        Tray11 = 30_u32.to_be(),
        Tray12 = 31_u32.to_be(),
        Tray13 = 32_u32.to_be(),
        Tray14 = 33_u32.to_be(),
        Tray15 = 34_u32.to_be(),
        Tray16 = 35_u32.to_be(),
        Tray17 = 36_u32.to_be(),
        Tray18 = 37_u32.to_be(),
        Tray19 = 38_u32.to_be(),
        Tray20 = 39_u32.to_be(),
        Roll1 = 40_u32.to_be(),
        Roll2 = 41_u32.to_be(),
        Roll3 = 42_u32.to_be(),
        Roll4 = 43_u32.to_be(),
        Roll5 = 44_u32.to_be(),
        Roll6 = 45_u32.to_be(),
        Roll7 = 46_u32.to_be(),
        Roll8 = 47_u32.to_be(),
        Roll9 = 48_u32.to_be(),
        Roll10 = 49_u32.to_be(),
    }

    #[allow(dead_code)]
    #[repr(u32)]
    pub enum Orientation {
        Portrait = 0_u32.to_be(),
        Landscape = 1_u32.to_be(),
        ReversePortrait = 2_u32.to_be(),
        ReverseLandscape = 3_u32.to_be(),
    }

    // 4.3.1.9
    #[allow(dead_code)]
    #[repr(u32)]
    pub enum PrintQuality {
        Default = 0_u32.to_be(),
        Draft = 3_u32.to_be(),
        Normal = 4_u32.to_be(),
        High = 5_u32.to_be(),
    }

    // 4.3.1.10
    pub struct Reserved<const N: usize>([u8; N]);

    impl<const N: usize> Default for Reserved<N> {
        fn default() -> Self {
            Self([0; N])
        }
    }

    // 4.3.1.11
    pub struct SrgbColor(u32);

    impl SrgbColor {
        pub fn new(value: impl Into<u32>) -> Self {
            Self(value.into().to_be())
        }
    }

    // 4.3.1.12
    pub struct UnsignedInteger(u32);

    impl UnsignedInteger {
        pub fn new(value: impl Into<u32>) -> Self {
            Self(value.into().to_be())
        }
    }

    impl Default for UnsignedInteger {
        fn default() -> Self {
            Self(0)
        }
    }

    // 4.3.1.13
    pub type VendorData = Reserved<1088>;

    // 4.3.1.14
    #[allow(dead_code)]
    #[repr(u32)]
    pub enum When {
        Never = 0_u32.to_be(),
        AfterDocument = 1_u32.to_be(),
        AfterJob = 2_u32.to_be(),
        AfterSet = 3_u32.to_be(),
        AfterPage = 4_u32.to_be(),
    }
}

// 4.3.2.1
#[allow(dead_code)]
struct PwgRaster(pub CString);

// 4.3.2.2
#[repr(C, packed)]
struct HwResolution {
    pub feed_res_dpi: UnsignedInteger,
    pub cross_feed_res_dpi: UnsignedInteger,
}

impl HwResolution {
    pub fn new(feed_dpi: u32, cross_feed_dpi: u32) -> Self {
        Self {
            feed_res_dpi: UnsignedInteger::new(feed_dpi),
            cross_feed_res_dpi: UnsignedInteger::new(cross_feed_dpi),
        }
    }
}

// 4.3.3.11
#[repr(C, packed)]
struct PageSize {
    pub width: UnsignedInteger,
    pub height: UnsignedInteger,
}

impl PageSize {
    pub fn new(page_pixels: &A4Pixels) -> Self {
        let width_dpi = page_pixels.resolution_width() as u64;
        let width_points = ((page_pixels.width() as u64) * 72 + width_dpi / 2) / width_dpi;

        let height_dpi = page_pixels.resolution_height() as u64;
        let height_points = ((page_pixels.height() as u64) * 72 + height_dpi / 2) / height_dpi;

        Self {
            width: UnsignedInteger::new(width_points.min(u32::MAX as u64) as u32),
            height: UnsignedInteger::new(height_points.min(u32::MAX as u64) as u32),
        }
    }
}

use types::*;

use crate::{error::Error, render::A4Pixels};

const PWG_SYNC_WORD: &str = "RaS2";
const PWG_RASTER: &str = "PwgRaster";
const ISO_A4_NAME: &str = "iso_a4_210x297mm";

#[repr(C, packed)]
#[allow(non_snake_case)]
struct PageHeader {
    PwgRaster: PwgRaster,
    MediaColor: CString,
    MediaType: CString,
    PrintContentOptimize: CString,
    Reserved1: Reserved<12>,
    CutMedia: When,
    Duplex: Boolean,
    HWResolution: HwResolution,
    Reserved2: Reserved<16>,
    InsertSheet: Boolean,
    Jog: When,
    LeadingEdge: Edge,
    Reserved3: Reserved<12>,
    MediaPosition: MediaPosition,
    MediaWeight: UnsignedInteger,
    Reserved4: Reserved<8>,
    NumCopies: UnsignedInteger,
    Orientation: Orientation,
    Reserved5: Reserved<4>,
    PageSize: PageSize,
    Reserved6: Reserved<8>,
    Tumble: Boolean,
    Width: UnsignedInteger,
    Height: UnsignedInteger,
    Reserved7: Reserved<4>,
    BitsPerColor: UnsignedInteger,
    BitsPerPixel: UnsignedInteger,
    BytesPerLine: UnsignedInteger,
    ColorOrder: ColorOrder,
    ColorSpace: ColorSpace,
    Reserved8: Reserved<16>,
    NumColors: UnsignedInteger,
    Reserved9: Reserved<28>,
    TotalPageCount: UnsignedInteger,
    CrossFeedTransform: Integer,
    FeedTransform: Integer,
    ImageBoxLeft: UnsignedInteger,
    ImageBoxTop: UnsignedInteger,
    ImageBoxRight: UnsignedInteger,
    ImageBoxBottom: UnsignedInteger,
    AlternatePrimary: SrgbColor,
    PrintQuality: PrintQuality,
    Reserved10: Reserved<20>,
    VendorIdentifier: UnsignedInteger,
    VendorLength: UnsignedInteger,
    VendorData: VendorData,
    Reserved11: Reserved<64>,
    RenderingIntent: CString,
    PageSizeName: CString,
}

impl PageHeader {
    pub fn new(page_pixels: &A4Pixels) -> Self {
        Self {
            PwgRaster: PwgRaster(CString::new(PWG_RASTER)),
            MediaColor: CString::default(),
            MediaType: CString::default(), // TODO
            PrintContentOptimize: CString::default(),
            Reserved1: Default::default(),
            CutMedia: When::Never,
            Duplex: Boolean::new(false),
            HWResolution: HwResolution::new(
                page_pixels.resolution_height() as u32,
                page_pixels.resolution_width() as u32,
            ),
            Reserved2: Default::default(),
            InsertSheet: Boolean::new(false),
            Jog: When::Never,
            LeadingEdge: Edge::ShortEdgeFirst, // TODO? likely not
            Reserved3: Default::default(),
            MediaPosition: MediaPosition::Auto,      // TODO
            MediaWeight: UnsignedInteger::default(), // TODO
            Reserved4: Default::default(),
            NumCopies: UnsignedInteger::new(1_u32), // TODO
            Orientation: Orientation::Portrait,
            Reserved5: Default::default(),
            PageSize: PageSize::new(page_pixels),
            Reserved6: Default::default(),
            Tumble: Boolean::new(false), // TODO?
            Width: UnsignedInteger::new(page_pixels.width() as u32),
            Height: UnsignedInteger::new(page_pixels.height() as u32),
            Reserved7: Default::default(),
            BitsPerColor: UnsignedInteger::new(page_pixels.bits_per_pixel() as u32),
            BitsPerPixel: UnsignedInteger::new(page_pixels.bits_per_pixel() as u32),
            BytesPerLine: UnsignedInteger::new(page_pixels.bytes_per_line() as u32),
            ColorOrder: ColorOrder::Chunky,
            ColorSpace: ColorSpace::Sgray,
            Reserved8: Default::default(),
            NumColors: UnsignedInteger::new(1_u32),
            Reserved9: Default::default(),
            TotalPageCount: UnsignedInteger::new(1_u32), // TODO
            CrossFeedTransform: Integer::new(1),
            FeedTransform: Integer::new(1),
            ImageBoxLeft: UnsignedInteger::default(),
            ImageBoxTop: UnsignedInteger::default(),
            ImageBoxRight: UnsignedInteger::default(),
            ImageBoxBottom: UnsignedInteger::default(),
            AlternatePrimary: SrgbColor::new(0x00ffffff_u32),
            PrintQuality: PrintQuality::Default, // TODO?
            Reserved10: Default::default(),
            VendorIdentifier: UnsignedInteger::default(),
            VendorLength: UnsignedInteger::default(),
            VendorData: Default::default(),
            Reserved11: Default::default(),
            RenderingIntent: CString::default(),
            PageSizeName: CString::new(ISO_A4_NAME), // TODO
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { from_raw_parts((self as *const Self) as *const u8, size_of::<Self>()) }
    }
}

pub(crate) fn write_file_header(_: &A4Pixels, writer: &mut impl Write) -> Result<(), Error> {
    writer.write_all(PWG_SYNC_WORD.as_bytes())?;
    Ok(())
}

pub(crate) fn write_page_header(pixels: &A4Pixels, writer: &mut impl Write) -> Result<(), Error> {
    writer.write_all(PageHeader::new(pixels).as_slice())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::PageHeader;

    pub const PWG_HEADER_SIZE: usize = 1796;

    #[test]
    fn test_page_size_matches() {
        assert_eq!(PWG_HEADER_SIZE, size_of::<PageHeader>())
    }
}
