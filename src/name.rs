// https://docs.microsoft.com/en-us/typography/opentype/spec/name

use std::convert::TryFrom;

use crate::parser::{Stream, FromData, SafeStream, LazyArray};
use crate::{Font, TableName, Result, Error};


/// A [platform ID](https://docs.microsoft.com/en-us/typography/opentype/spec/name#platform-ids).
#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(missing_docs)]
pub enum PlatformId {
    Unicode,
    Macintosh,
    Iso,
    Windows,
    Custom,
}

impl TryFrom<u16> for PlatformId {
    type Error = &'static str;

    #[inline]
    fn try_from(value: u16) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(PlatformId::Unicode),
            1 => Ok(PlatformId::Macintosh),
            2 => Ok(PlatformId::Iso),
            3 => Ok(PlatformId::Windows),
            4 => Ok(PlatformId::Custom),
            _ => Err("invalid id"),
        }
    }
}


#[inline]
fn is_unicode_encoding(platform_id: PlatformId, encoding_id: u16) -> bool {
    // https://docs.microsoft.com/en-us/typography/opentype/spec/name#windows-encoding-ids
    const WINDOWS_UNICODE_BMP_ENCODING_ID: u16 = 1;

    match platform_id {
        PlatformId::Unicode => true,
        PlatformId::Windows if encoding_id == WINDOWS_UNICODE_BMP_ENCODING_ID => true,
        _ => false,
    }
}


/// A [name ID](https://docs.microsoft.com/en-us/typography/opentype/spec/name#name-ids).
#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(missing_docs)]
pub enum NameId {
    CopyrightNotice,
    Family,
    Subfamily,
    UniqueID,
    FullName,
    Version,
    PostScriptName,
    Trademark,
    Manufacturer,
    Designer,
    Description,
    VendorUrl,
    DesignerUrl,
    License,
    LicenseUrl,
    TypographicFamily,
    TypographicSubfamily,
    CompatibleFull,
    SampleText,
    PostScriptCID,
    WWSFamily,
    WWSSubfamily,
    LightBackgroundPalette,
    DarkBackgroundPalette,
    VariationsPostScriptNamePrefix,
}

impl TryFrom<u16> for NameId {
    type Error = &'static str;

    #[inline]
    fn try_from(value: u16) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(NameId::CopyrightNotice),
            1 => Ok(NameId::Family),
            2 => Ok(NameId::Subfamily),
            3 => Ok(NameId::UniqueID),
            4 => Ok(NameId::FullName),
            5 => Ok(NameId::Version),
            6 => Ok(NameId::PostScriptName),
            7 => Ok(NameId::Trademark),
            8 => Ok(NameId::Manufacturer),
            9 => Ok(NameId::Designer),
            10 => Ok(NameId::Description),
            11 => Ok(NameId::VendorUrl),
            12 => Ok(NameId::DesignerUrl),
            13 => Ok(NameId::License),
            14 => Ok(NameId::LicenseUrl),
            // 15 - reserved
            16 => Ok(NameId::TypographicFamily),
            17 => Ok(NameId::TypographicSubfamily),
            18 => Ok(NameId::CompatibleFull),
            19 => Ok(NameId::SampleText),
            20 => Ok(NameId::PostScriptCID),
            21 => Ok(NameId::WWSFamily),
            22 => Ok(NameId::WWSSubfamily),
            23 => Ok(NameId::LightBackgroundPalette),
            24 => Ok(NameId::DarkBackgroundPalette),
            25 => Ok(NameId::VariationsPostScriptNamePrefix),
            _ => Err("invalid id"),
        }
    }
}


/// A [Name Record](https://docs.microsoft.com/en-us/typography/opentype/spec/name#name-records).
#[derive(Clone, Copy)]
pub struct Name<'a> {
    /// Raw name data.
    pub name: &'a [u8],

    /// Platform ID.
    pub platform_id: PlatformId,

    /// Platform-specific encoding ID.
    pub encoding_id: u16,

    /// Language ID.
    pub language_id: u16,

    /// Name ID.
    pub name_id: NameId,
}

impl<'a> Name<'a> {
    /// Converts Name's data into a `String`.
    ///
    /// Only Unicode names are supported. And since they are stored as UTF-16BE,
    /// we can't return `&str` and have to allocate a `String`.
    ///
    /// Supports:
    /// - Unicode Platform ID
    /// - Windows Platform ID + Unicode BMP
    #[inline(never)]
    pub fn to_string(&self) -> Option<String> {
        if self.is_unicode() {
            self.name_from_utf16_be()
        } else {
            None
        }
    }

    #[inline]
    fn is_unicode(&self) -> bool {
        is_unicode_encoding(self.platform_id, self.encoding_id)
    }

    #[inline(never)]
    fn name_from_utf16_be(&self) -> Option<String> {
        let mut name: Vec<u16> = Vec::new();
        for c in LazyArray::new(self.name) {
            name.push(c);
        }

        String::from_utf16(&name).ok()
    }
}

impl<'a> std::fmt::Debug for Name<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // TODO: https://github.com/rust-lang/rust/issues/50264

        let name = self.to_string();
        f.debug_struct("Name")
            .field("name", &name.as_ref().map(std::ops::Deref::deref)
                                .unwrap_or("unsupported encoding"))
            .field("platform_id", &self.platform_id)
            .field("encoding_id", &self.encoding_id)
            .field("language_id", &self.language_id)
            .field("name_id", &self.name_id)
            .finish()
    }
}


#[derive(Clone, Copy)]
struct NameRecord {
    platform_id: u16,
    encoding_id: u16,
    language_id: u16,
    name_id: u16,
    length: u16,
    offset: u16,
}

impl FromData for NameRecord {
    #[inline]
    fn parse(s: &mut SafeStream) -> Self {
        NameRecord {
            platform_id: s.read(),
            encoding_id: s.read(),
            language_id: s.read(),
            name_id: s.read(),
            length: s.read(),
            offset: s.read(),
        }
    }
}


/// An iterator over font's names.
#[derive(Clone, Copy)]
#[allow(missing_debug_implementations)]
pub struct Names<'a> {
    names: LazyArray<'a, NameRecord>,
    storage: &'a [u8],
    index: u16,
}

impl<'a> Iterator for Names<'a> {
    type Item = Name<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index as usize == self.names.len() {
            return None;
        }

        let index = self.index;
        self.index += 1;
        let name = self.names.get(index)?;

        let platform_id = match PlatformId::try_from(name.platform_id) {
            Ok(v) => v,
            Err(_) => return self.next(),
        };

        let name_id = match NameId::try_from(name.name_id) {
            Ok(v) => v,
            Err(_) => return self.next(),
        };

        let start = name.offset as usize;
        let end = start + name.length as usize;
        let data = match self.storage.get(start..end) {
            Some(data) => data,
            None => return self.next(),
        };

        Some(Name {
            name: data,
            platform_id,
            encoding_id: name.encoding_id,
            language_id: name.language_id,
            name_id,
        })
    }
}


impl<'a> Font<'a> {
    /// Returns an iterator over [Name Records].
    ///
    /// [Name Records]: https://docs.microsoft.com/en-us/typography/opentype/spec/name#name-records
    pub fn names(&self) -> Names {
        match self._names() {
            Ok(v) => v,
            Err(_) => Names { names: LazyArray::new(&[]), storage: &[], index: 0 },
        }
    }

    #[inline(never)]
    fn _names(&self) -> Result<Names> {
        // https://docs.microsoft.com/en-us/typography/opentype/spec/name#naming-table-format-1
        const LANG_TAG_RECORD_SIZE: u16 = 4;

        let data = self.name.ok_or_else(|| Error::TableMissing(TableName::Naming))?;
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        let count: u16 = s.read()?;
        s.skip::<u16>(); // offset
        let names = s.read_array(count)?;

        if format == 0 {
            Ok(Names {
                names,
                storage: s.tail()?,
                index: 0,
            })
        } else if format == 1 {
            let lang_tag_count: u16 = s.read()?;
            let lang_tag_len = lang_tag_count
                .checked_mul(LANG_TAG_RECORD_SIZE)
                .ok_or_else(|| Error::NotATrueType)?;

            s.skip_len(lang_tag_len); // langTagRecords
            Ok(Names {
                names,
                storage: s.tail()?,
                index: 0,
            })
        } else {
            // Invalid format.
            // The error type doesn't matter, since we will ignore it anyway.
            Err(Error::NotATrueType)
        }
    }

    /// Returns font's family name.
    ///
    /// Note that font can have multiple names. You can use [`names()`] to list them all.
    ///
    /// [`names()`]: #method.names
    pub fn family_name(&self) -> Option<String> {
        // Prefer Typographic Family name.

        let name = self.names()
            .find(|name| name.name_id == NameId::TypographicFamily && name.is_unicode())
            .and_then(|name| name.to_string());

        match name {
            Some(name) => return Some(name),
            None => {}
        }

        self.names()
            .find(|name| name.name_id == NameId::Family && name.is_unicode())
            .and_then(|name| name.to_string())
    }

    /// Returns font's PostScript name.
    ///
    /// Note that font can have multiple names. You can use [`names()`] to list them all.
    ///
    /// [`names()`]: #method.names
    pub fn post_script_name(&self) -> Option<String> {
        self.names()
            .find(|name| name.name_id == NameId::PostScriptName && name.is_unicode())
            .and_then(|name| name.to_string())
    }
}
