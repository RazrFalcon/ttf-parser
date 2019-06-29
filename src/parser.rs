use crate::Result;


pub trait FromData: Sized {
    /// Parses an object from a raw data.
    fn parse(s: &mut Stream) -> Self;

    /// Returns an object size in raw data.
    ///
    /// `mem::size_of` by default.
    ///
    /// Reimplement when size of `Self` != size of a raw data.
    /// For example, when you parsing u16, but storing it as u8.
    /// In this case `size_of::<Self>()` == 1, but `FromData::raw_size()` == 2.
    fn raw_size() -> usize {
        core::mem::size_of::<Self>()
    }
}

impl FromData for u8 {
    #[inline]
    fn parse(s: &mut Stream) -> Self {
        s.tail()[0]
    }
}

impl FromData for i8 {
    #[inline]
    fn parse(s: &mut Stream) -> Self {
        s.tail()[0] as i8
    }
}

impl FromData for u16 {
    #[inline]
    fn parse(s: &mut Stream) -> Self {
        let data = s.tail();
        (data[0] as u16) << 8 | data[1] as u16
    }
}

impl FromData for i16 {
    #[inline]
    fn parse(s: &mut Stream) -> Self {
        let data = s.tail();
        ((data[0] as u16) << 8 | data[1] as u16) as i16
    }
}

impl FromData for u32 {
    #[inline]
    fn parse(s: &mut Stream) -> Self {
        let data = s.tail();
        (data[0] as u32) << 24 | (data[1] as u32) << 16 | (data[2] as u32) << 8 | data[3] as u32
    }
}


pub trait TryFromData: Sized {
    /// Parses an object from a raw data.
    fn try_parse(s: &mut Stream) -> Result<Self>;

    /// Returns an object size in raw data.
    ///
    /// `mem::size_of` by default.
    ///
    /// Reimplement when size of `Self` != size of a raw data.
    /// For example, when you parsing u16, but storing it as u8.
    /// In this case `size_of::<Self>()` == 1, but `TryFromData::raw_size()` == 2.
    fn raw_size() -> usize {
        core::mem::size_of::<Self>()
    }
}


// Like `usize`, but for font.
pub trait FSize {
    fn to_usize(&self) -> usize;
}

impl FSize for u16 {
    #[inline]
    fn to_usize(&self) -> usize { *self as usize }
}

impl FSize for u32 {
    #[inline]
    fn to_usize(&self) -> usize { *self as usize }
}



#[derive(Clone, Copy)]
pub struct LazyArray<'a, T> {
    data: &'a [u8],
    phantom: core::marker::PhantomData<T>,
}

impl<'a, T: FromData> LazyArray<'a, T> {
    #[inline]
    pub fn new(data: &'a [u8]) -> Self {
        assert_eq!(data.len() % T::raw_size(), 0);

        LazyArray {
            data,
            phantom: core::marker::PhantomData,
        }
    }

    #[inline]
    pub fn at<L: FSize>(&self, index: L) -> T {
        self.get(index).unwrap()
    }

    pub fn get<L: FSize>(&self, index: L) -> Option<T> {
        if index.to_usize() < self.len() {
            let start = index.to_usize() * T::raw_size();
            let end = start + T::raw_size();
            let mut s = Stream::new(&self.data[start..end]);
            Some(T::parse(&mut s))
        } else {
            None
        }
    }

    #[inline]
    pub fn last(&self) -> T {
        self.at(self.len() as u32 - 1)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len() / T::raw_size()
    }

    #[inline]
    pub fn binary_search_by<F>(&self, mut f: F) -> Option<T>
        where F: FnMut(&T) -> core::cmp::Ordering
    {
        // Based on Rust std implementation.

        use core::cmp::Ordering;

        let mut size = self.len() as u32;
        if size == 0 {
            return None;
        }

        let mut base = 0;
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

impl<'a, T: FromData + core::fmt::Debug> core::fmt::Debug for LazyArray<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let array: LazyArrayIter<T> = LazyArrayIter {
            data: self.data,
            offset: 0,
            phantom: core::marker::PhantomData,
        };

        f.debug_list().entries(array).finish()
    }
}

impl<'a, T: FromData> IntoIterator for LazyArray<'a, T> {
    type Item = T;
    type IntoIter = LazyArrayIter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        LazyArrayIter {
            data: self.data,
            offset: 0,
            phantom: core::marker::PhantomData,
        }
    }
}


pub struct LazyArrayIter<'a, T> {
    data: &'a [u8],
    offset: usize,
    phantom: core::marker::PhantomData<T>,
}

impl<'a, T: FromData> Iterator for LazyArrayIter<'a, T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.offset;
        if offset == self.data.len() {
            return None;
        }

        self.offset += T::raw_size();
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
    pub fn jump_to_end(&mut self) {
        self.offset = self.data.len();
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
    pub fn skip<T: FromData>(&mut self) {
        self.offset += T::raw_size();
    }

    #[inline]
    pub fn skip_len<L: FSize>(&mut self, len: L) {
        self.offset += len.to_usize();
    }

    #[inline]
    pub fn read<T: FromData>(&mut self) -> T {
        let item = Self::read_at(self.data, self.offset);
        self.offset += T::raw_size();
        item
    }

    #[inline]
    pub fn try_read<T: TryFromData>(&mut self) -> Result<T> {
        let start = self.offset;
        self.offset += T::raw_size();
        let end = start + T::raw_size();
        let mut s = Stream::new(&self.data[start..end]);
        T::try_parse(&mut s)
    }

    #[inline]
    pub fn read_at<T: FromData>(data: &[u8], offset: usize) -> T {
        let start = offset;
        let end = start + T::raw_size();
        let mut s = Stream::new(&data[start..end]);
        T::parse(&mut s)
    }

    #[inline]
    pub fn read_bytes<L: FSize>(&mut self, len: L) -> &'a [u8] {
        let offset = self.offset;
        self.offset += len.to_usize();
        &self.data[offset..(offset + len.to_usize())]
    }

    #[inline]
    pub fn read_array<T: FromData, L: FSize>(&mut self, len: L) -> LazyArray<'a, T> {
        let len = len.to_usize() * T::raw_size();
        let array = LazyArray::new(&self.data[self.offset..(self.offset + len)]);
        self.offset += len;
        array
    }

    #[inline]
    pub fn read_u24(&mut self) -> u32 {
        let d = self.data;
        let n = 0 << 24 | (d[0] as u32) << 16 | (d[1] as u32) << 8 | d[2] as u32;
        self.offset += 3;
        n
    }

    pub fn read_f2_14(&mut self) -> f32 {
        self.read::<i16>() as f32 / 16384.0
    }
}


#[derive(Clone, Copy, Debug)]
pub struct Offset32(pub u32);

impl FromData for Offset32 {
    #[inline]
    fn parse(s: &mut Stream) -> Self {
        Offset32(s.read())
    }
}

impl FromData for Option<Offset32> {
    #[inline]
    fn parse(s: &mut Stream) -> Self {
        let offset: Offset32 = s.read();
        if offset.0 != 0 { Some(offset) } else { None }
    }

    fn raw_size() -> usize {
        Offset32::raw_size()
    }
}
