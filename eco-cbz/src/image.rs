use std::{
    io::{BufRead, Cursor, Read, Seek},
    path::Path,
};

use image::{io::Reader as ImageReader, DynamicImage, ImageFormat};
use zip::read::ZipFile;

use crate::errors::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReadingOrder {
    Rtl,
    Ltr,
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
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        let buf = Cursor::new(bytes);
        let reader = ImageReader::new(buf).with_guessed_format()?;
        let format = reader.format();
        Ok(Self {
            dynamic_image: reader.decode()?,
            format,
        })
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn try_from_zip_file(mut file: ZipFile<'_>) -> Result<Self> {
        #[allow(clippy::cast_possible_truncation)]
        let mut buf = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut buf)?;

        Self::try_from_bytes(&buf)
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

    #[allow(clippy::missing_errors_doc)]
    pub fn try_into_bytes(self) -> Result<Vec<u8>> {
        let mut buf = Cursor::new(Vec::new());
        self.dynamic_image
            .write_to(&mut buf, self.format.unwrap_or(ImageFormat::Png))?;
        Ok(buf.into_inner())
    }
}

impl TryFrom<Image> for Vec<u8> {
    type Error = Error;

    fn try_from(image: Image) -> Result<Self> {
        image.try_into_bytes()
    }
}

impl<'a> TryFrom<ZipFile<'a>> for Image {
    type Error = Error;

    fn try_from(file: ZipFile<'a>) -> Result<Self> {
        Self::try_from_zip_file(file)
    }
}

impl TryFrom<&[u8]> for Image {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        Self::try_from_bytes(bytes)
    }
}

impl TryFrom<Vec<u8>> for Image {
    type Error = Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self> {
        Self::try_from_bytes(&bytes)
    }
}
