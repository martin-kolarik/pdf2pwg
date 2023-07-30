use pdfium_render::prelude::PdfiumError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Compose(#[from] std::io::Error),
    #[error("{0}")]
    Render(#[from] PdfiumError),
}
