use std::sync::Arc;

use blocking::unblock;
use pdfium_render::prelude::{
    PdfBitmap, PdfBitmapFormat, PdfPageRenderRotation, PdfRenderConfig, Pdfium,
};

use crate::{error::Error, pwg, rle::compress, urf};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Portrait = 0,
    Landscape = 1,
}

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
    pub width: usize,
    pub resolution_width: usize,
    pub height: usize,
    pub resolution_height: usize,
    pub bits_per_pixel: usize,
}

impl A4Pixels {
    pub(crate) fn new(
        orientation: Orientation,
        resolution_width: Resolution,
        resolution_height: Resolution,
        colors: Colors,
    ) -> Self {
        let portrait_width = match resolution_width {
            Resolution::Dpi300 => 2480,
            Resolution::Dpi400 => 3307,
            Resolution::Dpi600 => 4960,
        };

        let portrait_height = match resolution_height {
            Resolution::Dpi300 => 3508,
            Resolution::Dpi400 => 4667,
            Resolution::Dpi600 => 7016,
        };

        let (width, height) = match orientation {
            Orientation::Portrait => (portrait_width, portrait_height),
            Orientation::Landscape => (portrait_height, portrait_width),
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

    pub fn bytes_per_line(&self) -> usize {
        (self.width * self.bits_per_pixel + 7) / 8
    }

    pub fn len(&self) -> usize {
        self.height * self.bytes_per_line()
    }
}

pub async fn render(
    pdf: Arc<Vec<u8>>,
    format: Format,
    orientation: Orientation,
    resolution_width: Resolution,
    resolution_height: Resolution,
) -> Result<Vec<u8>, Error> {
    unblock(move || {
        do_render(
            &pdf,
            format,
            orientation,
            resolution_width,
            resolution_height,
        )
    })
    .await
}

fn do_render(
    pdf: &[u8],
    format: Format,
    orientation: Orientation,
    resolution_width: Resolution,
    resolution_height: Resolution,
) -> Result<Vec<u8>, Error> {
    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
            .or_else(|_| Pdfium::bind_to_system_library())?,
    );
    let document = pdfium.load_pdf_from_byte_slice(&pdf, None)?;

    let color_a4 = A4Pixels::new(
        orientation,
        resolution_width,
        resolution_height,
        Colors::Colored,
    );

    let render_config = PdfRenderConfig::new()
        .set_target_size(color_a4.width as i32, color_a4.height as i32)
        .rotate_if_landscape(PdfPageRenderRotation::Degrees90, true)
        .use_grayscale_rendering(true)
        .set_text_smoothing(false)
        .use_print_quality(true);

    let mut color_bitmap = PdfBitmap::empty(
        color_a4.width as i32,
        color_a4.height as i32,
        PdfBitmapFormat::BGR,
        pdfium.bindings(),
    )?;

    let gray_a4 = A4Pixels::new(
        orientation,
        resolution_width,
        resolution_height,
        Colors::Gray,
    );
    let mut gray_bytes = vec![0u8; gray_a4.len()];

    let page_count = document.pages().len() as usize;
    let mut output = Vec::with_capacity(page_count * gray_a4.len() / 50);

    match format {
        Format::Pwg => pwg::write_file_header(&gray_a4, &mut output)?,
        Format::Urf => urf::write_file_header(&gray_a4, page_count as u32, &mut output)?,
    }

    for pdf_page in document.pages().iter() {
        match format {
            Format::Pwg => pwg::write_page_header(&gray_a4, &mut output)?,
            Format::Urf => urf::write_page_header(&gray_a4, &mut output)?,
        }

        pdf_page.render_into_bitmap_with_config(&mut color_bitmap, &render_config)?;

        color_bitmap
            .as_raw_bytes()
            .chunks(3)
            .enumerate()
            .for_each(|(index, pixel)| gray_bytes[index] = pixel[0]);

        compress(
            &gray_bytes,
            gray_a4.width,
            gray_a4.bits_per_pixel,
            &mut output,
        )?;
    }

    Ok(output)
}
