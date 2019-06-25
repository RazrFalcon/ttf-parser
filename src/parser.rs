use crate::GlyphId;

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

impl FromData for u8 {
    #[inline]
    fn parse(data: &[u8]) -> Self {
        data[0]
    }
}

impl FromData for u16 {
    #[inline]
    fn parse(data: &[u8]) -> Self {
        (data[0] as u16) << 8 | data[1] as u16
    }
}

impl FromData for i16 {
    #[inline]
    fn parse(data: &[u8]) -> Self {
        ((data[0] as u16) << 8 | data[1] as u16) as i16
    }
}

impl FromData for u32 {
    #[inline]
    fn parse(data: &[u8]) -> Self {
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

    #[inline]
    pub fn binary_search_by<F>(&self, mut f: F) -> Option<T>
        where F: FnMut(&T) -> std::cmp::Ordering
    {
        // Based on Rust std implementation.

        use std::cmp::Ordering;

        let mut size = self.len();
        if size == 0 {
            return None;
        }

        let mut base = 0usize;
        while size > 1 {
            let half = size / 2;
            let mid = base + half;
            // mid is always in [0, size), that means mid is >= 0 and < size.
            // mid >= 0: by definition
            // mid < size: mid = size / 2 + size / 4 + size / 8 ...
            let cmp = f(&self.at(mid));
            base = if cmp == Ordering::Greater { base } else { mid };
            size -= half;
        }

        // base is always in [0, size) because base <= mid.
        let value = self.at(base);
        let cmp = f(&value);
        if cmp == Ordering::Equal { Some(value) } else { None }
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
    pub fn read_u24(&mut self) -> u32 {
        let d = self.data;
        let n = 0 << 24 | (d[0] as u32) << 16 | (d[1] as u32) << 8 | d[2] as u32;
        self.offset += 3;
        n
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
        self.read()
    }
}


#[derive(Clone, Copy, Debug)]
pub struct Offset32(pub u32);

impl FromData for Offset32 {
    #[inline]
    fn parse(data: &[u8]) -> Self {
        Offset32(Stream::read_at(data, 0))
    }
}

impl FromData for Option<Offset32> {
    #[inline]
    fn parse(data: &[u8]) -> Self {
        let offset: Offset32 = Stream::read_at(data, 0);
        if offset.0 != 0 { Some(offset) } else { None }
    }

    fn size_of() -> usize {
        Offset32::size_of()
    }
}
