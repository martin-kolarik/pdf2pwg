use blocking::unblock;
use bytes::{BufMut, Bytes, BytesMut};
use pdfium_render::{
    prelude::{PdfBitmap, PdfBitmapFormat, PdfPageRenderRotation, Pdfium},
    render_config::PdfRenderConfig,
};

use crate::{error::Error, pwg, rle::compress, urf};

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum Resolution {
    Dpi300 = 300,
    Dpi400 = 400,
    Dpi600 = 600,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum Format {
    Pwg,
    Urf,
}

pub(crate) enum Colors {
    Gray,
    Colored,
}

pub(crate) struct A4Pixels {
    width: usize,
    resolution_width: usize,
    height: usize,
    resolution_height: usize,
    bits_per_pixel: usize,
}

impl A4Pixels {
    pub(crate) fn new(
        resolution_width: Resolution,
        resolution_height: Resolution,
        colors: Colors,
    ) -> Self {
        let width = match resolution_width {
            Resolution::Dpi300 => 2480,
            Resolution::Dpi400 => 3307,
            Resolution::Dpi600 => 4960,
        };

        let height = match resolution_height {
            Resolution::Dpi300 => 3508,
            Resolution::Dpi400 => 4667,
            Resolution::Dpi600 => 7016,
        };

        let bits_per_pixel = match colors {
            Colors::Gray => 8,
            Colors::Colored => 24,
        };

        Self {
            width,
            resolution_width: resolution_width as usize,
            height,
            resolution_height: resolution_height as usize,
            bits_per_pixel,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn width_i32(&self) -> i32 {
        self.width as i32
    }

    pub fn resolution_width(&self) -> usize {
        self.resolution_width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn height_i32(&self) -> i32 {
        self.height as i32
    }

    pub fn resolution_height(&self) -> usize {
        self.resolution_height
    }

    pub fn bits_per_pixel(&self) -> usize {
        self.bits_per_pixel
    }

    pub fn bytes_per_line(&self) -> usize {
        (self.width * self.bits_per_pixel + 7) / 8
    }

    pub fn len(&self) -> usize {
        self.height * self.bytes_per_line()
    }
}

pub async fn render(
    pdf: Bytes,
    resolution_width: Resolution,
    resolution_height: Resolution,
    format: Format,
) -> Result<Bytes, Error> {
    unblock(move || do_render(pdf, resolution_width, resolution_height, format)).await
}

fn do_render(
    pdf: Bytes,
    resolution_width: Resolution,
    resolution_height: Resolution,
    format: Format,
) -> Result<Bytes, Error> {
    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
            .or_else(|_| Pdfium::bind_to_system_library())?,
    );
    let document = pdfium.load_pdf_from_byte_slice(&pdf, None)?;

    let color_a4 = A4Pixels::new(resolution_width, resolution_height, Colors::Colored);

    let render_config = PdfRenderConfig::new()
        .set_target_size(color_a4.width_i32(), color_a4.height_i32())
        .rotate_if_landscape(PdfPageRenderRotation::Degrees90, true)
        .use_grayscale_rendering(true)
        .set_text_smoothing(false)
        .use_print_quality(true);

    let mut color_bitmap = PdfBitmap::empty(
        color_a4.width_i32(),
        color_a4.height_i32(),
        PdfBitmapFormat::BGR,
        pdfium.bindings(),
    )?;

    let gray_a4 = A4Pixels::new(resolution_width, resolution_height, Colors::Gray);
    let mut gray_bytes = vec![0u8; gray_a4.len()];

    let page_count = document.pages().len() as usize;
    let output = BytesMut::with_capacity(page_count * gray_a4.len() / 50);
    let mut writer = output.writer();

    match format {
        Format::Pwg => pwg::write_file_header(&gray_a4, &mut writer)?,
        Format::Urf => urf::write_file_header(&gray_a4, page_count as u32, &mut writer)?,
    }

    for pdf_page in document.pages().iter() {
        match format {
            Format::Pwg => pwg::write_page_header(&gray_a4, &mut writer)?,
            Format::Urf => urf::write_page_header(&gray_a4, &mut writer)?,
        }

        pdf_page.render_into_bitmap_with_config(&mut color_bitmap, &render_config)?;

        color_bitmap
            .as_raw_bytes()
            .chunks(3)
            .enumerate()
            .for_each(|(index, pixel)| gray_bytes[index] = pixel[0]);

        compress(
            &gray_bytes,
            gray_a4.width(),
            gray_a4.bits_per_pixel(),
            &mut writer,
        )?;
    }

    Ok(writer.into_inner().freeze())
}
