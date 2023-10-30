use std::{
    fmt::Display,
    io::{BufRead, Cursor, Seek},
    path::Path,
};

use image::{io::Reader as ImageReader, DynamicImage, ImageFormat};

use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum ReadingOrder {
    Rtl,
    Ltr,
}

impl Display for ReadingOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Ltr => "ltr",
                Self::Rtl => "rtl",
            }
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct Image {
    dynamic_image: DynamicImage,
    format: Option<ImageFormat>,
}

impl Image {
    /// ## Errors
    ///
    /// Fails if the image can't be open or decoded
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let reader = ImageReader::open(&path)?;
        let format = reader.format();
        Ok(Self {
            dynamic_image: reader.decode()?,
            format,
        })
    }

    /// ## Errors
    ///
    /// Fails if the image format can't be guessed or the image can't be decoded
    pub fn try_from_reader(reader: impl BufRead + Seek) -> Result<Self> {
        let reader = ImageReader::new(reader).with_guessed_format()?;
        let format = reader.format();
        Ok(Self {
            dynamic_image: reader.decode()?,
            format,
        })
    }

    /// ## Errors
    ///
    /// Fails if the image format can't be guessed or the image can't be decoded
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Self::try_from(bytes)
    }

    fn from_dynamic_image(dynamic_image: DynamicImage, format: Option<ImageFormat>) -> Self {
        Self {
            dynamic_image,
            format,
        }
    }

    #[must_use]
    pub fn is_portrait(&self) -> bool {
        self.dynamic_image.height() > self.dynamic_image.width()
    }

    #[must_use]
    pub fn is_landscape(&self) -> bool {
        !self.is_portrait()
    }

    #[must_use]
    pub fn set_contrast(self, contrast: f32) -> Self {
        Self::from_dynamic_image(self.dynamic_image.adjust_contrast(contrast), self.format)
    }

    #[must_use]
    pub fn set_brightness(self, brightness: i32) -> Self {
        Self::from_dynamic_image(self.dynamic_image.brighten(brightness), self.format)
    }

    #[must_use]
    pub fn set_blur(self, blur: f32) -> Self {
        Self::from_dynamic_image(self.dynamic_image.blur(blur), self.format)
    }

    #[must_use]
    pub fn autosplit(self, reading_order: ReadingOrder) -> (Image, Image) {
        let img1 = Self::from_dynamic_image(
            self.dynamic_image.crop_imm(
                0,
                0,
                self.dynamic_image.width() / 2,
                self.dynamic_image.height(),
            ),
            self.format,
        );
        let img2 = Self::from_dynamic_image(
            self.dynamic_image.crop_imm(
                self.dynamic_image.width() / 2,
                0,
                self.dynamic_image.width(),
                self.dynamic_image.height(),
            ),
            self.format,
        );
        match reading_order {
            ReadingOrder::Ltr => (img1, img2),
            ReadingOrder::Rtl => (img2, img1),
        }
    }

    #[must_use]
    pub fn dynamic(&self) -> &DynamicImage {
        &self.dynamic_image
    }

    #[must_use]
    pub fn format(&self) -> Option<ImageFormat> {
        self.format
    }

    pub fn set_format(&mut self, format: ImageFormat) -> &Self {
        self.format = Some(format);
        self
    }
}

impl TryFrom<&[u8]> for Image {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> std::result::Result<Self, Self::Error> {
        let buf_reader = Cursor::new(bytes);
        let reader = ImageReader::new(buf_reader).with_guessed_format()?;
        let format = reader.format();
        Ok(Self {
            dynamic_image: reader.decode()?,
            format,
        })
    }
}
