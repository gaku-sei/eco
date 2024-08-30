use std::{
    fs::File,
    io::{BufRead, BufReader, Cursor, Read, Seek},
    path::Path,
};

use image::{DynamicImage, ImageFormat, ImageReader};
use zip::read::ZipFile;

use crate::errors::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReadingOrder {
    Rtl,
    Ltr,
}

enum ImageInner<R: Read + Seek> {
    Reader(Option<ImageReader<R>>),
    DynamicImage(DynamicImage),
}

impl<R> ImageInner<R>
where
    R: BufRead + Seek,
{
    fn decode(&mut self) {
        if let Self::Reader(reader) = self {
            let reader = reader.take().unwrap();
            *self = Self::DynamicImage(reader.decode().unwrap());
        }
    }

    fn dynamic_image(&self) -> Option<&DynamicImage> {
        if let Self::DynamicImage(dynamic_image) = self {
            Some(dynamic_image)
        } else {
            None
        }
    }
}

impl<R: Read + Seek> From<DynamicImage> for ImageInner<R> {
    fn from(dynamic_image: DynamicImage) -> Self {
        Self::DynamicImage(dynamic_image)
    }
}

impl<R: Read + Seek> From<ImageReader<R>> for ImageInner<R> {
    fn from(reader: ImageReader<R>) -> Self {
        Self::Reader(Some(reader))
    }
}

pub struct Image<R: Read + Seek> {
    format: ImageFormat,
    inner: ImageInner<R>,
}

#[allow(clippy::module_name_repetitions)]
pub type ImageBuf = Image<Cursor<Vec<u8>>>;

#[allow(clippy::module_name_repetitions)]
pub type ImageBytes<'a> = Image<Cursor<&'a [u8]>>;

#[allow(clippy::module_name_repetitions)]
pub type ImageFile = Image<BufReader<File>>;

impl Image<BufReader<File>> {
    /// ## Errors
    ///
    /// Fails if the image can't be open or decoded
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let reader = ImageReader::open(&path)?.with_guessed_format()?;
        let Some(format) = reader.format() else {
            return Err(Error::UnknownImageFormat);
        };

        Ok(Self {
            inner: reader.into(),
            format,
        })
    }
}

impl<'a> Image<Cursor<&'a [u8]>> {
    /// ## Errors
    ///
    /// Fails if the image format can't be guessed
    pub fn try_from_bytes(bytes: &'a [u8]) -> Result<Self> {
        let reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
        let Some(format) = reader.format() else {
            return Err(Error::UnknownImageFormat);
        };

        Ok(Self {
            inner: reader.into(),
            format,
        })
    }

    #[must_use]
    pub fn bytes_with_format(bytes: &'a [u8], format: ImageFormat) -> Self {
        let mut reader = ImageReader::new(Cursor::new(bytes));
        reader.set_format(format);

        Self {
            inner: reader.into(),
            format,
        }
    }
}

impl Image<Cursor<Vec<u8>>> {
    /// ## Errors
    ///
    /// Fails if the image format can't be guessed
    pub fn try_from_buf(buf: Vec<u8>) -> Result<Self> {
        let reader = ImageReader::new(Cursor::new(buf)).with_guessed_format()?;
        let Some(format) = reader.format() else {
            return Err(Error::UnknownImageFormat);
        };

        Ok(Self {
            inner: reader.into(),
            format,
        })
    }

    #[must_use]
    pub fn buf_with_format(buf: Vec<u8>, format: ImageFormat) -> Self {
        let mut reader = ImageReader::new(Cursor::new(buf));
        reader.set_format(format);

        Self {
            inner: reader.into(),
            format,
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn try_from_zip_file(mut file: ZipFile<'_>) -> Result<Self> {
        #[allow(clippy::cast_possible_truncation)]
        let mut buf = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut buf)?;
        Self::try_from_buf(buf)
    }
}

impl<R> Image<R>
where
    R: BufRead + Seek,
{
    /// ## Errors
    ///
    /// Fails if the image format can't be guessed
    pub fn try_from_reader(reader: R) -> Result<Self> {
        let reader = ImageReader::new(reader).with_guessed_format()?;
        let Some(format) = reader.format() else {
            return Err(Error::UnknownImageFormat);
        };

        Ok(Self {
            inner: reader.into(),
            format,
        })
    }

    fn from_dynamic_image(dynamic_image: DynamicImage, format: ImageFormat) -> Self {
        Self {
            inner: dynamic_image.into(),
            format,
        }
    }

    #[must_use]
    pub fn is_portrait(&mut self) -> bool {
        let image = self.image();
        image.height() > image.width()
    }

    #[must_use]
    pub fn is_landscape(&mut self) -> bool {
        !self.is_portrait()
    }

    #[must_use]
    pub fn set_contrast(mut self, contrast: f32) -> Self {
        Self::from_dynamic_image(self.image().adjust_contrast(contrast), self.format)
    }

    #[must_use]
    pub fn set_brightness(mut self, brightness: i32) -> Self {
        Self::from_dynamic_image(self.image().brighten(brightness), self.format)
    }

    #[must_use]
    pub fn set_blur(mut self, blur: f32) -> Self {
        Self::from_dynamic_image(self.image().blur(blur), self.format)
    }

    #[must_use]
    pub fn autosplit(mut self, reading_order: ReadingOrder) -> (Image<R>, Image<R>) {
        let format = self.format;
        let image = self.image();
        let height = image.height();
        let width = image.width();
        let img_width = width / 2;

        let img1 = Self::from_dynamic_image(image.crop_imm(0, 0, img_width, height), format);
        let img2 = Self::from_dynamic_image(image.crop_imm(img_width, 0, width, height), format);
        match reading_order {
            ReadingOrder::Ltr => (img1, img2),
            ReadingOrder::Rtl => (img2, img1),
        }
    }

    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn image(&mut self) -> &DynamicImage {
        self.inner.decode();
        self.inner.dynamic_image().unwrap()
    }

    #[must_use]
    pub fn format(&self) -> ImageFormat {
        self.format
    }

    pub fn set_format(&mut self, format: ImageFormat) -> &Self {
        self.format = format;
        self
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn try_into_bytes(self) -> Result<Vec<u8>> {
        match self.inner {
            ImageInner::Reader(Some(reader)) => {
                let mut buf = Vec::new();
                reader.into_inner().read_to_end(&mut buf)?;
                Ok(buf)
            }
            ImageInner::DynamicImage(dynamic_image) => {
                let mut buf = Cursor::new(Vec::new());
                let format = self.format;

                dynamic_image.write_to(&mut buf, format)?;
                Ok(buf.into_inner())
            }
            ImageInner::Reader(None) => unreachable!(),
        }
    }
}

impl<R> TryFrom<Image<R>> for Vec<u8>
where
    R: BufRead + Seek,
{
    type Error = Error;

    fn try_from(image: Image<R>) -> Result<Self> {
        image.try_into_bytes()
    }
}

impl<'a> TryFrom<ZipFile<'a>> for Image<Cursor<Vec<u8>>> {
    type Error = Error;

    fn try_from(file: ZipFile<'a>) -> Result<Self> {
        Self::try_from_zip_file(file)
    }
}

impl TryFrom<Vec<u8>> for Image<Cursor<Vec<u8>>> {
    type Error = Error;

    fn try_from(buf: Vec<u8>) -> Result<Self> {
        Self::try_from_buf(buf)
    }
}

impl<'a> TryFrom<&'a [u8]> for Image<Cursor<&'a [u8]>> {
    type Error = Error;

    fn try_from(bytes: &'a [u8]) -> Result<Self> {
        Self::try_from_bytes(bytes)
    }
}
