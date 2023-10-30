#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::{
    fs::{File, OpenOptions},
    io::{self, Cursor, Read, Seek, Write},
    marker::PhantomData,
    path::Path,
    result,
};

use ::image::ImageFormat;
use camino::Utf8Path;
use image::Image;
use tracing::debug;
use zip::{read::ZipFile, write::FileOptions, ZipArchive, ZipWriter};

pub use crate::errors::{Error, Result};

pub mod errors;
pub mod image;

/// We artificially limit the amount of accepted files to 65535 files per Cbz
/// First as it'd be rather impractical for the user to read such enormous Cbz
/// Also, this size has been chosen as it was the limit of the very first zip spec
pub static MAX_FILE_NUMBER: usize = u16::MAX as usize;

/// The length of 65535 used to name the inserted file with a proper padding
static COUNTER_SIZE: usize = 5;

pub struct CbzFile<'a>(ZipFile<'a>);

impl<'a> CbzFile<'a> {
    pub fn name(&self) -> &str {
        self.0.name()
    }

    pub fn size(&self) -> u64 {
        self.0.size()
    }

    pub fn raw_file(&self) -> &ZipFile<'a> {
        &self.0
    }

    pub fn raw_file_mut(&mut self) -> &mut ZipFile<'a> {
        &mut self.0
    }

    /// Convert the file content to bytes
    ///
    /// ## Errors
    ///
    /// Fails if file size is too large to fit a `usize` on host machine
    /// or if the content can't be read
    pub fn to_bytes(&mut self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(
            self.size()
                .try_into()
                .map_err(|_| Error::CbzFileSizeConversion)?,
        );

        self.0.read_to_end(&mut buf)?;

        Ok(buf)
    }
}

impl<'a> Read for CbzFile<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl<'a> From<ZipFile<'a>> for CbzFile<'a> {
    fn from(zip_file: ZipFile<'a>) -> Self {
        Self(zip_file)
    }
}

impl<'a> From<CbzFile<'a>> for ZipFile<'a> {
    fn from(cbz_file: CbzFile<'a>) -> Self {
        cbz_file.0
    }
}

#[derive(Debug)]
pub struct CbzReader<'a, R> {
    archive: ZipArchive<R>,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a, R> CbzReader<'a, R> {
    pub fn new(archive: ZipArchive<R>) -> Self {
        Self {
            archive,
            _lifetime: PhantomData,
        }
    }

    pub fn archive(&self) -> &ZipArchive<R> {
        &self.archive
    }

    pub fn archive_mut(&mut self) -> &mut ZipArchive<R> {
        &mut self.archive
    }
}

impl<'a, R> CbzReader<'a, R>
where
    R: Read + Seek,
{
    pub fn len(&self) -> usize {
        self.archive.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn file_names(&self) -> impl Iterator<Item = &str> {
        self.archive.file_names()
    }

    /// Lookup the file by `name` in Cbz and returns a `CbzFile`
    ///
    /// ## Errors
    ///
    /// Fails if file size is too large to fit a `usize` on host machine
    /// or if the content can't be read
    pub fn read_by_name(&mut self, name: &str) -> Result<CbzFile<'_>> {
        let archive_file = self.archive.by_name(name)?;

        Ok(archive_file.into())
    }

    pub fn for_each<F>(&mut self, mut f: F)
    where
        F: FnMut(Result<CbzFile<'_>>),
    {
        let mut file_names = self.file_names().map(Into::into).collect::<Vec<String>>();
        file_names.sort();

        for file_name in file_names {
            f(self.read_by_name(&file_name));
        }
    }

    /// Iterate over files present in the Cbz.
    /// If the closure returns an error, this error is returned immediately.
    ///
    /// ## Errors
    ///
    /// Returns an error immediately if the provided closure returns an error
    pub fn try_for_each<F, E>(&mut self, mut f: F) -> result::Result<(), E>
    where
        F: FnMut(Result<CbzFile<'_>>) -> result::Result<(), E>,
    {
        let mut file_names = self.file_names().map(Into::into).collect::<Vec<String>>();
        file_names.sort();

        for file_name in file_names {
            f(self.read_by_name(&file_name))?;
        }

        Ok(())
    }

    /// Creates `CbzReader` from a `Read`
    ///
    /// ## Errors
    ///
    /// Fails if the underlying `ZipArchive` can't be created
    pub fn try_from_reader(reader: R) -> Result<Self> {
        let archive = ZipArchive::new(reader)?;

        Ok(Self::new(archive))
    }
}

impl<'a> CbzReader<'a, File> {
    /// Creates `CbzReader` from a path
    ///
    /// ## Errors
    ///
    /// Fails if the underlying `ZipArchive` can't be created
    pub fn try_from_path(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path.as_ref())?;

        Self::try_from_reader(file)
    }
}

impl<'a, 'b> CbzReader<'a, Cursor<&'b [u8]>> {
    /// Creates `CbzReader` from a bytes slice
    ///
    /// ## Errors
    ///
    /// Fails if the underlying `ZipArchive` can't be created
    pub fn try_from_bytes_slice(bytes: &'b [u8]) -> Result<Self> {
        let cursor = Cursor::new(bytes);

        Self::try_from_reader(cursor)
    }
}

impl<'a> CbzReader<'a, Cursor<Vec<u8>>> {
    /// Creates `CbzReader` from bytes
    ///
    /// ## Errors
    ///
    /// Fails if the underlying `ZipArchive` can't be created
    pub fn try_from_bytes(bytes: impl Into<Vec<u8>>) -> Result<Self> {
        let cursor = Cursor::new(bytes.into());

        Self::try_from_reader(cursor)
    }
}

impl<'a, R> From<ZipArchive<R>> for CbzReader<'a, R> {
    fn from(archive: ZipArchive<R>) -> Self {
        Self::new(archive)
    }
}

impl<'a, R> From<CbzReader<'a, R>> for ZipArchive<R> {
    fn from(cbz: CbzReader<'a, R>) -> Self {
        cbz.archive
    }
}

pub struct CbzWriter<'a, W: Write + Seek> {
    archive: ZipWriter<W>,
    size: usize,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a, W> CbzWriter<'a, W>
where
    W: Write + Seek,
{
    pub fn new(archive: ZipWriter<W>) -> Self {
        Self {
            archive,
            size: 0,
            _lifetime: PhantomData,
        }
    }

    /// Creates a `CbzWriter` from a `Write`
    fn from_writer(writer: W) -> Self {
        let archive = ZipWriter::new(writer);

        Self::new(archive)
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn raw_archive(&self) -> &ZipWriter<W> {
        &self.archive
    }

    pub fn raw_archive_mut(&mut self) -> &mut ZipWriter<W> {
        &mut self.archive
    }

    /// ## Errors
    ///
    /// Same behavior as `insert_with_file_options`
    pub fn insert(&mut self, bytes: &[u8], extension: &str) -> Result<()> {
        self.insert_with_file_options(bytes, extension, FileOptions::default())
    }

    /// ## Errors
    ///
    /// Same behavior as `insert_with_file_options`
    pub fn insert_cbz_file(&mut self, file: CbzFile, extension: &str) -> Result<()> {
        let bytes = file
            .0
            .bytes()
            .map(|res| res.map_err(Into::into))
            .collect::<Result<Vec<_>>>()?;
        self.insert_with_file_options(&bytes, extension, FileOptions::default())
    }

    /// ## Errors
    ///
    /// Same behavior as `insert_with_file_options`
    pub fn insert_image(&mut self, image: &Image) -> Result<()> {
        let mut out = Cursor::new(Vec::new());
        let format = image.format().unwrap_or(ImageFormat::Png);
        image.dynamic().write_to(&mut out, format)?;
        self.insert_with_file_options(
            out.get_ref(),
            format.extensions_str()[0],
            FileOptions::default(),
        )
    }

    /// ## Errors
    ///
    /// This fails if the Cbz writer can't be written or if it's full (i.e. its size equals `MAX_FILE_NUMBER`)
    pub fn insert_with_file_options(
        &mut self,
        bytes: &[u8],
        extension: &str,
        file_options: FileOptions,
    ) -> Result<()> {
        self.insert_from_bytes_slice_with_options(bytes, extension, file_options)
    }

    /// This is the method ultimately called to insert the bytes into the Cbz
    ///
    /// ## Errors
    ///
    /// This fails if the Cbz writer can't be written or if it's full (i.e. its size equals `MAX_FILE_NUMBER`)
    fn insert_from_bytes_slice_with_options(
        &mut self,
        bytes: &[u8],
        extension: &str,
        file_options: FileOptions,
    ) -> Result<()> {
        let filename = format!("{:0>COUNTER_SIZE$}.{}", self.len() + 1, extension);

        if self.size >= MAX_FILE_NUMBER {
            return Err(Error::CbzTooLarge(MAX_FILE_NUMBER));
        }

        self.archive.start_file(filename, file_options)?;
        self.archive.write_all(bytes)?;
        self.size += 1;

        Ok(())
    }
}

impl<'a> CbzWriter<'a, Cursor<Vec<u8>>> {
    /// ## Errors
    ///
    /// Same errors as the underlying `ZipWriter::finish` method
    pub fn write_to(mut self, mut writer: impl Write) -> Result<()> {
        writer.write_all(&self.archive.finish()?.into_inner())?;

        Ok(())
    }

    /// Writes self into a File (that will be created) located under the provided path
    ///
    /// ## Errors
    ///
    /// Can fail on file creation or when writing the file content
    pub fn write_to_path(self, path: impl AsRef<Utf8Path>) -> Result<()> {
        let path = path.as_ref();
        debug!("writing cbz file to {path}");
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(
                path.with_file_name(
                    path.file_name()
                        .map(sanitize_filename::sanitize)
                        .unwrap_or_default(),
                ),
            )?;
        self.write_to(&mut file)
    }
}

impl<'a> Default for CbzWriter<'a, Cursor<Vec<u8>>> {
    fn default() -> Self {
        Self::from_writer(Cursor::new(Vec::new()))
    }
}

impl<'a, W> From<ZipWriter<W>> for CbzWriter<'a, W>
where
    W: Write + Seek,
{
    fn from(archive: ZipWriter<W>) -> Self {
        Self::new(archive)
    }
}

impl<'a, W> From<CbzWriter<'a, W>> for ZipWriter<W>
where
    W: Write + Seek,
{
    fn from(cbz: CbzWriter<'a, W>) -> Self {
        cbz.archive
    }
}
