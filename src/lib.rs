#![feature(slice_group_by)]

mod pdfium;
pub use pdfium::{render, Resolution};

mod pwg_header;
pub(crate) use pwg_header::PageHeader;

mod pwg_rle;
