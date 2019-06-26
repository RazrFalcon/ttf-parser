#[cfg(feature = "std")]
use std::vec::Vec;
#[cfg(feature = "std")]
use std::string::String;

use core::convert::TryFrom;

use crate::parser::Stream;
use crate::{Font, TableName};


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

    fn try_from(value: u16) -> Result<Self, Self::Error> {
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

    fn try_from(value: u16) -> Result<Self, Self::Error> {
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
#[cfg_attr(not(feature = "std"), derive(Debug))]
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
    #[cfg(feature = "std")]
    pub fn to_string(&self) -> Option<String> {
        if self.is_supported_encoding() {
            self.name_from_utf16_be()
        } else {
            None
        }
    }

    #[cfg(feature = "std")]
    fn is_supported_encoding(&self) -> bool {
        // https://docs.microsoft.com/en-us/typography/opentype/spec/name#windows-encoding-ids
        const WINDOWS_UNICODE_BMP_ENCODING_ID: u16 = 1;

        match self.platform_id {
            PlatformId::Unicode => true,
            PlatformId::Windows if self.encoding_id == WINDOWS_UNICODE_BMP_ENCODING_ID => true,
            _ => false,
        }
    }

    #[cfg(feature = "std")]
    fn name_from_utf16_be(&self) -> Option<String> {
        use crate::parser::LazyArray;

        let mut name: Vec<u16> = Vec::new();
        for c in LazyArray::new(self.name) {
            name.push(c);
        }

        String::from_utf16(&name).ok()
    }
}

#[cfg(feature = "std")]
impl<'a> core::fmt::Debug for Name<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        // TODO: https://github.com/rust-lang/rust/issues/50264

        let name = self.to_string();
        f.debug_struct("Name")
            .field("name", &name.as_ref().map(core::ops::Deref::deref)
                                .unwrap_or("unsupported encoding"))
            .field("platform_id", &self.platform_id)
            .field("encoding_id", &self.encoding_id)
            .field("language_id", &self.language_id)
            .field("name_id", &self.name_id)
            .finish()
    }
}


/// An iterator over font's names.
#[derive(Clone, Copy)]
#[allow(missing_debug_implementations)]
pub struct Names<'a> {
    stream: Stream<'a>,
    storage: &'a [u8],
}

impl<'a> Iterator for Names<'a> {
    type Item = Name<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stream.at_end() {
            return None;
        }

        let platform_id = PlatformId::try_from(self.stream.read::<u16>());
        let encoding_id: u16 = self.stream.read();
        let language_id: u16 = self.stream.read();
        let name_id = NameId::try_from(self.stream.read::<u16>());
        let length = self.stream.read::<u16>() as usize;
        let offset = self.stream.read::<u16>() as usize;

        let platform_id = match platform_id {
            Ok(v) => v,
            Err(_) => return self.next(),
        };

        let name_id = match name_id {
            Ok(v) => v,
            Err(_) => return self.next(),
        };

        Some(Name {
            name: &self.storage[offset..(offset + length)],
            platform_id,
            encoding_id,
            language_id,
            name_id,
        })
    }
}


impl<'a> Font<'a> {
    /// Returns an iterator over [Name Records].
    ///
    /// [Name Records]: https://docs.microsoft.com/en-us/typography/opentype/spec/name#name-records
    pub fn names(&self) -> Names {
        // https://docs.microsoft.com/en-us/typography/opentype/spec/name#name-records
        const NAME_RECORD_SIZE: u16 = 12;

        // https://docs.microsoft.com/en-us/typography/opentype/spec/name#naming-table-format-1
        const LANG_TAG_RECORD_SIZE: u16 = 4;

        let data = match self.table_data(TableName::Naming) {
            Ok(data) => data,
            Err(_) => return Names { stream: Stream::new(&[]), storage: &[] },
        };

        let mut s = Stream::new(data);
        let format: u16 = s.read();
        let count: u16 = s.read();
        s.skip::<u16>(); // offset
        let name_record_len = count * NAME_RECORD_SIZE;
        let name_records_data = s.read_bytes(name_record_len);

        if format == 0 {
            Names {
                stream: Stream::new(name_records_data),
                storage: s.tail(),
            }
        } else if format == 1 {
            let lang_tag_count: u16 = s.read();
            s.skip_len(lang_tag_count * LANG_TAG_RECORD_SIZE); // langTagRecords
            Names {
                stream: Stream::new(name_records_data),
                storage: s.tail(),
            }
        } else {
            // Invalid format.
            Names { stream: Stream::new(&[]), storage: &[] }
        }
    }

    /// Returns font's family name.
    ///
    /// Note that font can have multiple names. You can use [`names()`] to list them all.
    ///
    /// [`names()`]: #method.names
    #[cfg(feature = "std")]
    pub fn family_name(&self) -> Option<String> {
        // Prefer Typographic Family name.

        let name = self.names()
            .find(|name| name.name_id == NameId::TypographicFamily && name.is_supported_encoding())
            .and_then(|name| name.to_string());

        match name {
            Some(name) => return Some(name),
            None => {}
        }

        self.names()
            .find(|name| name.name_id == NameId::Family && name.is_supported_encoding())
            .and_then(|name| name.to_string())
    }

    /// Returns font's PostScript name.
    ///
    /// Note that font can have multiple names. You can use [`names()`] to list them all.
    ///
    /// [`names()`]: #method.names
    #[cfg(feature = "std")]
    pub fn post_stript_name(&self) -> Option<String> {
        self.names()
            .find(|name| name.name_id == NameId::PostScriptName && name.is_supported_encoding())
            .and_then(|name| name.to_string())
    }
}
