use crate::{Tag, GlyphId};

pub trait FromData: Sized {
    /// Parses an object from a raw data.
    fn parse(data: &[u8]) -> Self;

    /// Returns an object size in raw data.
    ///
    /// `mem::size_of` by default.
    ///
    /// Reimplement, when size of `Self` != size of raw data.
    /// For example, when you parsing u16, but storing it as u8.
    /// In this case `size_of::<Self>()` == 1, but `FromData::size_of()` == 2.
    fn size_of() -> usize {
        std::mem::size_of::<Self>()
    }
}

impl FromData for u16 {
    fn parse(data: &[u8]) -> u16 {
        (data[0] as u16) << 8 | data[1] as u16
    }
}

impl FromData for i16 {
    fn parse(data: &[u8]) -> i16 {
        ((data[0] as u16) << 8 | data[1] as u16) as i16
    }
}

impl FromData for u32 {
    fn parse(data: &[u8]) -> u32 {
        (data[0] as u32) << 24 | (data[1] as u32) << 16 | (data[2] as u32) << 8 | data[3] as u32
    }
}


pub struct LazyArray<'a, T> {
    data: &'a [u8],
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T: FromData> LazyArray<'a, T> {
    #[inline]
    pub fn new(data: &'a [u8]) -> Self {
        assert_eq!(data.len() % T::size_of(), 0);

        LazyArray {
            data,
            phantom: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn at(&self, index: usize) -> T {
        self.get(index).unwrap()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<T> {
        if index < self.len() {
            let start = index * T::size_of();
            let end = start + T::size_of();
            Some(T::parse(&self.data[start..end]))
        } else {
            None
        }
    }

    #[inline]
    pub fn last(&self) -> T {
        self.at(self.len() - 1)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len() / T::size_of()
    }
}

impl<'a, T: FromData> IntoIterator for LazyArray<'a, T> {
    type Item = T;
    type IntoIter = ArrayIter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ArrayIter {
            data: self.data,
            offset: 0,
            phantom: std::marker::PhantomData,
        }
    }
}

pub struct ArrayIter<'a, T> {
    data: &'a [u8],
    offset: usize,
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T: FromData> Iterator for ArrayIter<'a, T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.offset;
        if offset == self.data.len() {
            return None;
        }

        self.offset += T::size_of();
        Some(Stream::read_at(self.data, offset))
    }
}


#[derive(Clone, Copy)]
pub struct Stream<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Stream<'a> {
    #[inline]
    pub fn new(data: &'a [u8]) -> Self {
        Stream {
            data,
            offset: 0,
        }
    }

    #[inline]
    pub fn at_end(&self) -> bool {
        self.offset == self.data.len()
    }

    #[inline]
    pub fn offset(&self) -> usize {
        self.offset
    }

    #[inline]
    pub fn tail(&self) -> &'a [u8] {
        &self.data[self.offset..]
    }

    #[inline]
    pub fn skip(&mut self, len: usize) {
        self.offset += len;
    }

    #[inline]
    fn skip_item<T: FromData>(&mut self) {
        self.offset += T::size_of();
    }

    #[inline]
    pub fn skip_u16(&mut self) {
        self.skip_item::<u16>()
    }

    #[inline]
    pub fn skip_u32(&mut self) {
        self.skip_item::<u32>()
    }

    #[inline]
    pub fn read<T: FromData>(&mut self) -> T {
        let item = Self::read_at(self.data, self.offset);
        self.offset += T::size_of();
        item
    }

    #[inline]
    pub fn read_at<T: FromData>(data: &[u8], offset: usize) -> T {
        let start = offset;
        let end = start + T::size_of();
        T::parse(&data[start..end])
    }

    #[inline]
    pub fn read_bytes(&mut self, len: usize) -> &'a [u8] {
        let offset = self.offset;
        self.offset += len;
        &self.data[offset..(offset + len)]
    }

    #[inline]
    pub fn read_array<T: FromData>(&mut self, len: usize) -> LazyArray<'a, T> {
        let len = len * T::size_of();
        let array = LazyArray::new(&self.data[self.offset..(self.offset + len)]);
        self.offset += len;
        array
    }

    #[inline]
    pub fn read_u8(&mut self) -> u8 {
        self.offset += 1;
        self.data[self.offset - 1]
    }

    #[inline]
    pub fn read_u16(&mut self) -> u16 {
        self.read()
    }

    #[inline]
    pub fn read_u32(&mut self) -> u32 {
        self.read()
    }

    #[inline]
    pub fn read_i8(&mut self) -> i8 {
        self.offset += 1;
        self.data[self.offset - 1] as i8
    }

    #[inline]
    pub fn read_i16(&mut self) -> i16 {
        self.read()
    }

    #[inline]
    pub fn read_f2_14(&mut self) -> f32 {
        self.read_i16() as f32 / 16384.0
    }

    #[inline]
    pub fn read_glyph_id(&mut self) -> GlyphId {
        GlyphId(self.read())
    }

    #[inline]
    pub fn read_to(&mut self, slice: &mut [u8]) {
        for i in 0..slice.len() {
            slice[i] = self.read_u8();
        }
    }

    #[inline]
    pub fn read_tag(&mut self) -> Tag {
        let mut tag = Tag::zero();
        self.read_to(&mut tag.tag);
        tag
    }
}
