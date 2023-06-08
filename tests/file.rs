use std::{fs, sync::Arc};

use pdf2pwg::{render, Resolution};
use pdfium_render::prelude::PdfiumError;

#[async_std::test]
async fn render_file() -> Result<(), PdfiumError> {
    let pdf = fs::read(r"C:\Work\DancesportServices\pdf2pwg\tests\test.pdf").unwrap();
    let pages = render(Arc::new(pdf), Resolution::Dpi600, Resolution::Dpi600).await?;

    for (index, page) in pages.iter().enumerate() {
        fs::write(
            format!(r"C:\Work\DancesportServices\pdf2pwg\target\debug\page-{index}.pwg"),
            &page,
        )
        .unwrap();
    }

    Ok(())
}
