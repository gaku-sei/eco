#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::{
    fs::{File, OpenOptions},
    io::{Cursor, Read, Seek, Write},
    path::Path,
};

use camino::Utf8Path;
use tracing::debug;
use zip::{write::FileOptions, ZipArchive, ZipWriter};

pub use crate::errors::{Error, Result};
use crate::image::Image;

/// We artificially limit the amount of accepted files to 65535 files per Cbz
/// First as it'd be rather impractical for the user to read such enormous Cbz
/// Also, this size has been chosen as it was the limit of the very first zip spec
pub static MAX_FILE_NUMBER: usize = u16::MAX as usize;

/// The length of 65535 used to name the inserted file with a proper padding
static COUNTER_SIZE: usize = 5;

#[derive(Debug)]
pub struct Reader<R> {
    archive: ZipArchive<R>,
}

impl<R> Reader<R> {
    pub fn new(archive: ZipArchive<R>) -> Self {
        Self { archive }
    }

    pub fn archive(&self) -> &ZipArchive<R> {
        &self.archive
    }

    pub fn archive_mut(&mut self) -> &mut ZipArchive<R> {
        &mut self.archive
    }
}

impl<R> Reader<R>
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

    /// Lookup the image by `name` in Cbz and returns an `Image`
    ///
    /// ## Errors
    ///
    /// Fails if file size is too large to fit a `usize` on host machine
    /// or if the content can't be read
    pub fn read_by_name(&mut self, name: &str) -> Result<Image> {
        let file = self.archive.by_name(name)?;
        file.try_into()
    }

    /// Iterate over images present in the Cbz.
    pub fn for_each<F>(&mut self, mut f: F)
    where
        F: FnMut(Result<Image>),
    {
        // We need the extra allocations since `read_by_name` takes a mut ref
        let mut file_names = self.file_names().map(Into::into).collect::<Vec<String>>();
        file_names.sort();

        for file_name in file_names {
            f(self.read_by_name(&file_name));
        }
    }

    /// Iterate over images present in the Cbz.
    /// If the closure returns an error, this error is returned immediately.
    ///
    /// ## Errors
    ///
    /// Returns an error immediately if the provided closure returns an error
    pub fn try_for_each<F, E>(&mut self, mut f: F) -> Result<(), E>
    where
        F: FnMut(Result<Image>) -> Result<(), E>,
    {
        let mut file_names = self.file_names().map(Into::into).collect::<Vec<String>>();
        file_names.sort();

        for file_name in file_names {
            f(self.read_by_name(&file_name))?;
        }

        Ok(())
    }

    /// Creates `Reader` from a `Read`
    ///
    /// ## Errors
    ///
    /// Fails if the underlying `ZipArchive` can't be read
    pub fn try_from_reader(reader: R) -> Result<Self> {
        let archive = ZipArchive::new(reader)?;

        Ok(Self::new(archive))
    }

    /// Retrieves the metadata from the cbz file.
    /// The format has never been specified so any deserializable type is accepted.
    ///
    /// Some "official" doc can be found here: `https://web.archive.org/web/20110428152605/http://code.google.com/p/comicbookinfo/`.
    ///
    /// An example of what it may look like can be found here: `https://web.archive.org/web/20120429060204/http://code.google.com/p/comicbookinfo/wiki/Example`.
    ///
    /// ## Errors
    ///
    /// Fails if the underlying archive comment cannot be read or if the value doesn't match the provided type.
    #[cfg(feature = "metadata")]
    pub fn metadata<T>(&self) -> Result<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let mut comment = self.archive.comment();
        // https://web.archive.org/web/20110428152605/http://code.google.com/p/comicbookinfo/
        let mut buf = vec![0; u16::MAX as usize];
        let s = comment.read(&mut buf)?;

        // Drop last byte
        Ok(serde_json::from_slice(&buf[..s])?)
    }
}

impl Reader<File> {
    /// Creates a `Reader` from a path
    ///
    /// ## Errors
    ///
    /// Fails if the underlying `ZipArchive` can't be created
    pub fn try_from_path(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path.as_ref())?;

        Self::try_from_reader(file)
    }
}

impl<'a> Reader<Cursor<&'a [u8]>> {
    /// Creates `Reader` from a bytes slice
    ///
    /// ## Errors
    ///
    /// Fails if the underlying `ZipArchive` can't be created
    pub fn try_from_bytes_slice(bytes: &'a [u8]) -> Result<Self> {
        let cursor = Cursor::new(bytes);

        Self::try_from_reader(cursor)
    }
}

impl Reader<Cursor<Vec<u8>>> {
    /// Creates `Reader` from bytes
    ///
    /// ## Errors
    ///
    /// Fails if the underlying `ZipArchive` can't be created
    pub fn try_from_bytes(bytes: impl Into<Vec<u8>>) -> Result<Self> {
        let cursor = Cursor::new(bytes.into());

        Self::try_from_reader(cursor)
    }
}

impl<R> From<ZipArchive<R>> for Reader<R> {
    fn from(archive: ZipArchive<R>) -> Self {
        Self::new(archive)
    }
}

impl<R> From<Reader<R>> for ZipArchive<R> {
    fn from(cbz: Reader<R>) -> Self {
        cbz.archive
    }
}

pub struct Writer<W: Write + Seek> {
    archive: ZipWriter<W>,
    size: usize,
}

impl<W> Writer<W>
where
    W: Write + Seek,
{
    pub fn new(archive: ZipWriter<W>) -> Self {
        Self { archive, size: 0 }
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
    /// Same behavior as `insert_with_extension_and_file_options`
    pub fn insert(&mut self, image: Image) -> Result<()> {
        let extension = image
            .format()
            .and_then(|f| f.extensions_str().first().copied())
            .unwrap_or("png");
        self.insert_with_extension_and_file_options(image, extension, FileOptions::default())
    }

    /// ## Errors
    ///
    /// Same behavior as `insert_with_extension_and_file_options`
    pub fn insert_with_extension(&mut self, image: Image, extension: &str) -> Result<()> {
        self.insert_with_extension_and_file_options(image, extension, FileOptions::default())
    }

    /// ## Errors
    ///
    /// Same behavior as `insert_with_extension_and_file_options`
    pub fn insert_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        let image = bytes.try_into()?;
        self.insert(image)
    }

    /// ## Errors
    ///
    /// Same behavior as `insert_with_extension_and_file_options`
    pub fn insert_bytes_with_extension(&mut self, bytes: &[u8], extension: &str) -> Result<()> {
        let image = bytes.try_into()?;
        self.insert_with_extension(image, extension)
    }

    /// ## Errors
    ///
    /// Same behavior as `insert_with_extension_and_file_options`
    pub fn insert_bytes_with_extension_and_file_options(
        &mut self,
        bytes: &[u8],
        extension: &str,
        file_options: FileOptions,
    ) -> Result<()> {
        let image = bytes.try_into()?;
        self.insert_with_extension_and_file_options(image, extension, file_options)
    }

    /// ## Errors
    ///
    /// This fails if the Cbz writer can't be written or if it's full (i.e. its size equals `MAX_FILE_NUMBER`)
    pub fn insert_with_extension_and_file_options(
        &mut self,
        image: Image,
        extension: &str,
        file_options: FileOptions,
    ) -> Result<()> {
        if self.size >= MAX_FILE_NUMBER {
            return Err(Error::CbzTooLarge(MAX_FILE_NUMBER));
        }

        let filename = format!("{:0>COUNTER_SIZE$}.{}", self.len() + 1, extension);

        self.archive.start_file(filename, file_options)?;
        self.archive.write_all(&image.try_into_bytes()?)?;
        self.size += 1;

        Ok(())
    }

    /// Set the metadata of the cbz file.
    /// The format has never been specified so any serializable type is accepted.
    ///
    /// Some "official" doc can be found here: `https://web.archive.org/web/20110428152605/http://code.google.com/p/comicbookinfo/`.
    ///
    /// And an example of what it may look like can be found here: `https://web.archive.org/web/20120429060204/http://code.google.com/p/comicbookinfo/wiki/Example`.
    ///
    /// ## Errors
    ///
    /// Fails if the underlying archive comment cannot be written to,
    /// or if the provided metadata's json representation is bigger than 65,535 bytes.
    #[cfg(feature = "metadata")]
    pub fn set_metadata<T>(&mut self, metadata: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        let metadata = serde_json::to_string(metadata)?;
        if metadata.len() > u16::MAX as usize {
            return Err(Error::CbzMetadataSize(metadata.len()));
        }

        self.archive.set_comment(metadata);

        Ok(())
    }
}

impl Writer<Cursor<Vec<u8>>> {
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

impl Default for Writer<Cursor<Vec<u8>>> {
    fn default() -> Self {
        Self::from_writer(Cursor::new(Vec::new()))
    }
}

impl<W> From<ZipWriter<W>> for Writer<W>
where
    W: Write + Seek,
{
    fn from(archive: ZipWriter<W>) -> Self {
        Self::new(archive)
    }
}

impl<W> From<Writer<W>> for ZipWriter<W>
where
    W: Write + Seek,
{
    fn from(cbz: Writer<W>) -> Self {
        cbz.archive
    }
}
