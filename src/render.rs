use std::sync::Arc;

use async_std::task::spawn_blocking;
use pdfium_render::{
    prelude::{PdfBitmap, PdfBitmapFormat, PdfPageRenderRotation, Pdfium, PdfiumError},
    render_config::PdfRenderConfig,
};

use crate::{pwg, rle::compress_bitmap, urf};

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum Resolution {
    Dpi300 = 300,
    Dpi400 = 400,
    Dpi600 = 600,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum Output {
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
        self.resolution_width as usize
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn height_i32(&self) -> i32 {
        self.height as i32
    }

    pub fn resolution_height(&self) -> usize {
        self.resolution_height as usize
    }

    pub fn bits_per_pixel(&self) -> usize {
        self.bits_per_pixel
    }

    pub fn bytes_per_line(&self) -> usize {
        (self.width * self.bits_per_pixel + 7) / 8
    }

    pub fn len(&self) -> usize {
        self.height as usize * self.bytes_per_line() as usize
    }
}

pub async fn render(
    pdf: Arc<Vec<u8>>,
    resolution_width: Resolution,
    resolution_height: Resolution,
    output: Output,
) -> Result<Arc<Vec<Vec<u8>>>, PdfiumError> {
    spawn_blocking(move || do_render(pdf, resolution_width, resolution_height, output)).await
}

fn do_render(
    pdf: Arc<Vec<u8>>,
    resolution_width: Resolution,
    resolution_height: Resolution,
    output: Output,
) -> Result<Arc<Vec<Vec<u8>>>, PdfiumError> {
    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
            .or_else(|_| Pdfium::bind_to_system_library())?,
    );
    let document = pdfium.load_pdf_from_byte_slice(pdf.as_slice(), None)?;

    let rendered_pixels = A4Pixels::new(resolution_width, resolution_height, Colors::Colored);

    let render_config = PdfRenderConfig::new()
        .set_target_size(rendered_pixels.width_i32(), rendered_pixels.height_i32())
        .rotate_if_landscape(PdfPageRenderRotation::Degrees90, true)
        .use_grayscale_rendering(true)
        .set_text_smoothing(false)
        .use_print_quality(true);

    let mut rendered_color = PdfBitmap::empty(
        rendered_pixels.width_i32(),
        rendered_pixels.height_i32(),
        PdfBitmapFormat::BGR,
        pdfium.bindings(),
    )?;

    let compressed_pixels = A4Pixels::new(resolution_width, resolution_height, Colors::Gray);
    let mut compressed: Vec<u8> = Vec::with_capacity(compressed_pixels.len());

    let page_count = document.pages().len() as usize;
    let mut pages = Vec::with_capacity(page_count);
    for pdf_page in document.pages().iter() {
        pdf_page.render_into_bitmap_with_config(&mut rendered_color, &render_config)?;

        let rendered_color_bytes = rendered_color.as_bytes();

        let mut rendered_gray = vec![0u8; rendered_color_bytes.len() / 3];
        rendered_color_bytes
            .chunks(rendered_pixels.bits_per_pixel() / 8)
            .enumerate()
            .for_each(|(index, pixel)| rendered_gray[index] = pixel[0]);

        let _ = compress_bitmap(
            &rendered_gray,
            compressed_pixels.width(),
            compressed_pixels.bits_per_pixel(),
            &mut compressed,
        );

        pages.push(match output {
            Output::Pwg => pwg::create_page(&compressed_pixels, &compressed),
            Output::Urf => urf::create_page(&compressed_pixels, page_count as u32, &compressed),
        });

        compressed.clear();
    }
    Ok(Arc::new(pages))
}
