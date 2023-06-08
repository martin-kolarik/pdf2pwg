use std::{io::Write, mem::size_of, slice::from_raw_parts, sync::Arc};

use async_std::task::spawn_blocking;
use image::{
    buffer::{ConvertBuffer, Pixels},
    DynamicImage, EncodableLayout, GrayImage, ImageBuffer, Luma, Pixel, Rgb,
};
use pdfium_render::{
    prelude::{
        PdfBitmap, PdfBitmapFormat, PdfBitmapRotation, PdfPageRenderRotation, Pdfium, PdfiumError,
    },
    render_config::PdfRenderConfig,
};

use crate::{
    pwg_header::{PageHeader, PWG_SYNC_WORD},
    pwg_rle::compress_bitmap,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    Dpi300,
    Dpi400,
    Dpi600,
}

pub(crate) enum Colors {
    Gray,
    Colored,
}

pub(crate) struct A4Pixels {
    width: usize,
    height: usize,
    bits_per_pixel: usize,
}

impl A4Pixels {
    pub fn new(
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
            height,
            bits_per_pixel,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn width_i32(&self) -> i32 {
        self.width as i32
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn height_i32(&self) -> i32 {
        self.height as i32
    }

    pub fn bits_per_pixel(&self) -> usize {
        self.bits_per_pixel
    }

    pub fn bytes_per_line(&self) -> usize {
        (self.width + 7) * self.bits_per_pixel / 8
    }

    pub fn len(&self) -> usize {
        self.height as usize * self.bytes_per_line() as usize
    }
}

pub async fn render(
    pdf: Arc<Vec<u8>>,
    resolution_width: Resolution,
    resolution_height: Resolution,
) -> Result<Arc<Vec<Vec<u8>>>, PdfiumError> {
    spawn_blocking(move || do_render(pdf, resolution_width, resolution_height)).await
}

fn do_render(
    pdf: Arc<Vec<u8>>,
    resolution_width: Resolution,
    resolution_height: Resolution,
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
        .use_print_quality(true);

    let mut rendered_color = PdfBitmap::empty(
        rendered_pixels.width_i32(),
        rendered_pixels.height_i32(),
        PdfBitmapFormat::BGR,
        pdfium.bindings(),
    )?;

    let compressed_pixels = A4Pixels::new(resolution_width, resolution_height, Colors::Gray);
    let mut compressed: Vec<u8> = Vec::with_capacity(compressed_pixels.len());

    let mut pages = Vec::with_capacity(document.pages().len() as usize);
    for pdf_page in document.pages().iter() {
        pdf_page.render_into_bitmap_with_config(&mut rendered_color, &render_config)?;

        eprintln!("X1");

        let rendered_collor_buffer: ImageBuffer<Rgb<u8>, _> = ImageBuffer::from_raw(
            rendered_color.width() as u32,
            rendered_color.height() as u32,
            rendered_color.as_bytes(),
        )
        .unwrap(); // TODO error

        eprintln!("X2");

        let mut rendered_gray = vec![0u8; rendered_collor_buffer.len() / 3];
        rendered_color
            .as_bytes()
            .chunks(rendered_pixels.bits_per_pixel() / 8)
            .enumerate()
            .for_each(|(index, pixel)| rendered_gray[index] = pixel[0]);

        // let rendered_gray: GrayImage = rendered_collor_buffer.convert(); // TODO: slow in debug
        // rendered_gray
        //     .save_with_format("test-gray-converted.png", image::ImageFormat::Png) // ... and saves it to a file.
        //     .map_err(|_| PdfiumError::ImageError)?;

        eprintln!("X3");

        let _ = compress_bitmap(
            rendered_gray.as_bytes(),
            compressed_pixels.width(),
            compressed_pixels.bits_per_pixel(),
            &mut compressed,
        );

        eprintln!("X4");

        let mut page =
            Vec::with_capacity(PWG_SYNC_WORD.len() + size_of::<PageHeader>() + compressed.len());
        page.extend_from_slice(PWG_SYNC_WORD.as_bytes());
        page.extend_from_slice(
            PageHeader::new(
                compressed_pixels.height() as u32,
                compressed_pixels.width() as u32,
            )
            .as_slice(),
        );
        page.extend_from_slice(&compressed);

        eprintln!("X5");

        pages.push(page.clone());

        compressed.clear();
    }
    Ok(Arc::new(pages))
}
