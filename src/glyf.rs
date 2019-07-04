// This module is a heavily modified version of https://github.com/raphlinus/font-rs

use crate::parser::{Stream, LazyArray, TrySlice};
use crate::{Font, GlyphId, OutlineBuilder, TableName, Result};


/// A wrapper that transforms segments before passing them to `OutlineBuilder`.
trait OutlineBuilderInner {
    fn push_move_to(&mut self, x: f32, y: f32);
    fn push_line_to(&mut self, x: f32, y: f32);
    fn push_quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32);
    fn push_close(&mut self);
}

struct Builder<'a, T: OutlineBuilder> {
    builder: &'a mut T,
    transform: Transform,
    is_default_ts: bool, // `bool` is faster than `Option` or `is_default`.
}

impl<'a, T: OutlineBuilder> OutlineBuilderInner for Builder<'a, T> {
    fn push_move_to(&mut self, mut x: f32, mut y: f32) {
        if !self.is_default_ts {
            self.transform.apply_to(&mut x, &mut y);
        }

        self.builder.move_to(x, y);
    }

    fn push_line_to(&mut self, mut x: f32, mut y: f32) {
        if !self.is_default_ts {
            self.transform.apply_to(&mut x, &mut y);
        }

        self.builder.line_to(x, y);
    }

    fn push_quad_to(&mut self, mut x1: f32, mut y1: f32, mut x: f32, mut y: f32) {
        if !self.is_default_ts {
            self.transform.apply_to(&mut x1, &mut y1);
            self.transform.apply_to(&mut x, &mut y);
        }

        self.builder.quad_to(x1, y1, x, y);
    }

    fn push_close(&mut self) {
        self.builder.close();
    }
}


// https://docs.microsoft.com/en-us/typography/opentype/spec/glyf#simple-glyph-description
#[derive(Clone, Copy)]
struct SimpleGlyphFlags(u8);

impl SimpleGlyphFlags {
    const ON_CURVE_POINT: Self                          = Self(1 << 0);
    const X_SHORT_VECTOR: Self                          = Self(1 << 1);
    const Y_SHORT_VECTOR: Self                          = Self(1 << 2);
    const REPEAT_FLAG: Self                             = Self(1 << 3);
    const X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR: Self    = Self(1 << 4);
    const Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR: Self    = Self(1 << 5);

    #[inline] fn empty() -> Self { Self(0) }
    #[inline] fn all() -> Self { Self(63) }
    #[inline] fn from_bits_truncate(bits: u8) -> Self { Self(bits & Self::all().0) }
    #[inline] fn contains(&self, other: Self) -> bool { (self.0 & other.0) == other.0 }
}


// https://docs.microsoft.com/en-us/typography/opentype/spec/glyf#composite-glyph-description
#[derive(Clone, Copy)]
struct CompositeGlyphFlags(u16);

impl CompositeGlyphFlags {
    const ARG_1_AND_2_ARE_WORDS: Self     = Self(1 << 0);
    const ARGS_ARE_XY_VALUES: Self        = Self(1 << 1);
    const WE_HAVE_A_SCALE: Self           = Self(1 << 3);
    const MORE_COMPONENTS: Self           = Self(1 << 5);
    const WE_HAVE_AN_X_AND_Y_SCALE: Self  = Self(1 << 6);
    const WE_HAVE_A_TWO_BY_TWO: Self      = Self(1 << 7);

    #[inline] fn all() -> Self { Self(235) }
    #[inline] fn from_bits_truncate(bits: u16) -> Self { Self(bits & Self::all().0) }
    #[inline] fn contains(&self, other: Self) -> bool { (self.0 & other.0) == other.0 }
}


#[inline]
fn f32_bound(min: f32, val: f32, max: f32) -> f32 {
    debug_assert!(min.is_finite());
    debug_assert!(val.is_finite());
    debug_assert!(max.is_finite());

    if val > max {
        return max;
    } else if val < min {
        return min;
    }

    val
}

// It's not defined in the spec, so we are using our own value.
const MAX_COMPONENTS: u8 = 32;

impl<'a> Font<'a> {
    pub(crate) fn glyf_glyph_outline(
        &self,
        glyph_id: GlyphId,
        builder: &mut impl OutlineBuilder,
    ) -> Result<()> {
        let mut b = Builder {
            builder,
            transform: Transform::default(),
            is_default_ts: true,
        };

        let glyph_data = self.glyph_data(glyph_id)?;
        self.outline_impl(glyph_data, 0, &mut b)
    }

    fn glyph_data(&self, glyph_id: GlyphId) -> Result<&[u8]> {
        let range = self.glyph_range(glyph_id)?;
        let data = self.table_data(TableName::GlyphData)?;
        data.try_slice(range)
    }

    fn outline_impl<T: OutlineBuilder>(
        &self,
        data: &[u8],
        depth: u8,
        builder: &mut Builder<T>,
    ) -> Result<()> {
        if depth >= MAX_COMPONENTS {
            return Ok(());
        }

        let mut s = Stream::new(data);

        let number_of_contours: i16 = s.read()?;
        s.skip_len(8_u32); // skip bbox

        if number_of_contours > 0 {
            Self::parse_simple_outline(s.tail()?, number_of_contours as u16, builder)
        } else if number_of_contours < 0 {
            self.parse_composite_outline(s.tail()?, depth, builder)
        } else {
            // An empty glyph.
            Ok(())
        }
    }

    #[inline(never)]
    fn parse_simple_outline<T: OutlineBuilder>(
        glyph_data: &[u8],
        number_of_contours: u16,
        builder: &mut Builder<T>,
    ) -> Result<()> {
        let mut s = Stream::new(glyph_data);
        let endpoints: LazyArray<u16> = s.read_array(number_of_contours)?;

        let points_total = {
            // Unwrap is safe, because it's guarantee that array has at least one value.
            let last_point = endpoints.last().unwrap();
            // Prevent overflow.
            if last_point == std::u16::MAX {
                return Ok(());
            }

            last_point + 1
        };

        let instructions_len: u16 = s.read()?;
        s.skip_len(instructions_len);

        let flags_offset = s.offset();
        let x_coords_len = Self::resolve_x_coords_len(&mut s, points_total)?;
        let x_coords_offset = s.offset();
        let y_coords_offset = x_coords_offset + x_coords_len as usize;

        let mut points = GlyphPoints {
            flags: Stream::new(glyph_data.try_slice(flags_offset..x_coords_offset)?),
            x_coords: Stream::new(glyph_data.try_slice(x_coords_offset..y_coords_offset)?),
            y_coords: Stream::new(glyph_data.try_slice(y_coords_offset..glyph_data.len())?),
            points_left: points_total,
            flag_repeats: 0,
            last_flags: SimpleGlyphFlags::empty(),
            x: 0,
            y: 0,
        };

        let mut total = 0u16;
        let mut last = 0u16;
        for n in endpoints {
            if n < last {
                // Endpoints must be in increasing order.
                break;
            }
            last = n;

            // Check for overflow.
            if n == std::u16::MAX {
                break;
            }

            let n = n + 1 - total;

            // Contour must have at least 2 points.
            if n >= 2 {
                Self::parse_contour(points.by_ref().take(n as usize), builder);
            }

            total += n;
        }

        Ok(())
    }

    /// Resolves the X coordinates length.
    ///
    /// The length depends on *Simple Glyph Flags*, so we have to process them all to find it.
    fn resolve_x_coords_len(
        s: &mut Stream,
        points_total: u16,
    ) -> Result<u16> {
        type Flags = SimpleGlyphFlags;

        let mut flags_left = points_total;
        let mut x_coords_len = 0;
        while flags_left > 0 {
            let flags = Flags::from_bits_truncate(s.read()?);

            // The number of times a glyph point repeats.
            let repeats = if flags.contains(Flags::REPEAT_FLAG) {
                s.read::<u8>()? as u16 + 1
            } else {
                1
            };

            if flags.contains(Flags::X_SHORT_VECTOR) {
                // Coordinate is 1 byte long.
                x_coords_len += repeats;
            } else {
                if !flags.contains(Flags::X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR) {
                    // Coordinate is 2 bytes long.
                    x_coords_len += repeats * 2;
                }
            }

            // Check for overflow.
            if repeats > flags_left {
                // Not sure what should be done in this case.
                flags_left = 0;
            } else {
                flags_left -= repeats;
            }
        }

        Ok(x_coords_len)
    }

    fn parse_contour<T: OutlineBuilder>(
        points: std::iter::Take<&mut GlyphPoints>,
        builder: &mut Builder<T>,
    ) {
        let mut first_oncurve: Option<Point> = None;
        let mut first_offcurve: Option<Point> = None;
        let mut last_offcurve: Option<Point> = None;
        for point in points {
            let p = Point { x: point.x as f32, y: point.y as f32 };
            if first_oncurve.is_none() {
                if point.on_curve_point {
                    first_oncurve = Some(p);
                    builder.push_move_to(p.x, p.y);
                } else {
                    match first_offcurve {
                        Some(offcurve) => {
                            let mid = offcurve.lerp(p, 0.5);
                            first_oncurve = Some(mid);
                            last_offcurve = Some(p);
                            builder.push_move_to(mid.x, mid.y);
                        }
                        None => {
                            first_offcurve = Some(p);
                        }
                    }
                }
            } else {
                match (last_offcurve, point.on_curve_point) {
                    (Some(offcurve), true) => {
                        last_offcurve = None;
                        builder.push_quad_to(offcurve.x, offcurve.y, p.x, p.y);
                    }
                    (Some(offcurve), false) => {
                        last_offcurve = Some(p);
                        let mid = offcurve.lerp(p, 0.5);
                        builder.push_quad_to(offcurve.x, offcurve.y, mid.x, mid.y);
                    }
                    (None, true) => {
                        builder.push_line_to(p.x, p.y);
                    }
                    (None, false) => {
                        last_offcurve = Some(p);
                    }
                }
            }
        }

        loop {
            match (first_offcurve, last_offcurve) {
                (Some(offcurve1), Some(offcurve2)) => {
                    last_offcurve = None;
                    let mid = offcurve2.lerp(offcurve1, 0.5);
                    builder.push_quad_to(offcurve2.x, offcurve2.y, mid.x, mid.y);
                }
                (Some(offcurve1), None) => {
                    if let Some(p) = first_oncurve {
                        builder.push_quad_to(offcurve1.x, offcurve1.y, p.x, p.y);
                    }
                    break;
                }
                (None, Some(offcurve2)) => {
                    if let Some(p) = first_oncurve {
                        builder.push_quad_to(offcurve2.x, offcurve2.y, p.x, p.y);
                    }
                    break;
                }
                (None, None) => {
                    if let Some(p) = first_oncurve {
                        builder.push_line_to(p.x, p.y);
                    }
                    break;
                }
            }
        }

        builder.push_close();
    }

    #[inline(never)]
    fn parse_composite_outline<T: OutlineBuilder>(
        &self,
        glyph_data: &[u8],
        depth: u8,
        builder: &mut Builder<T>,
    ) -> Result<()> {
        type Flags = CompositeGlyphFlags;

        let mut s = Stream::new(glyph_data);
        let flags = Flags::from_bits_truncate(s.read()?);
        let glyph_id: GlyphId = s.read()?;

        let mut ts = Transform::default();

        if flags.contains(Flags::ARGS_ARE_XY_VALUES) {
            if flags.contains(Flags::ARG_1_AND_2_ARE_WORDS) {
                ts.e = s.read::<i16>()? as f32;
                ts.f = s.read::<i16>()? as f32;
            } else {
                ts.e = s.read::<i8>()? as f32;
                ts.f = s.read::<i8>()? as f32;
            }
        }

        if flags.contains(Flags::WE_HAVE_A_TWO_BY_TWO) {
            ts.a = s.read_f2_14()?;
            ts.b = s.read_f2_14()?;
            ts.c = s.read_f2_14()?;
            ts.d = s.read_f2_14()?;
        } else if flags.contains(Flags::WE_HAVE_AN_X_AND_Y_SCALE) {
            ts.a = s.read_f2_14()?;
            ts.d = s.read_f2_14()?;
        } else if flags.contains(Flags::WE_HAVE_A_SCALE) {
            ts.a = f32_bound(-2.0, s.read_f2_14()?, 2.0);
            ts.d = ts.a;
        }

        if let Ok(glyph_data) = self.glyph_data(glyph_id) {
            let transform = Transform::combine(builder.transform, ts);
            let mut b = Builder {
                builder: builder.builder,
                transform,
                is_default_ts: transform.is_default(),
            };

            self.outline_impl(glyph_data, depth + 1, &mut b)?;
        }

        if flags.contains(Flags::MORE_COMPONENTS) {
            if depth <= MAX_COMPONENTS {
                self.parse_composite_outline(s.tail()?, depth + 1, builder)?;
            }
        }

        Ok(())
    }
}


#[derive(Clone, Copy)]
struct Transform {
    a: f32, b: f32, c: f32,
    d: f32, e: f32, f: f32,
}

impl Transform {
    fn combine(ts1: Self, ts2: Self) -> Self {
        Transform {
            a: ts1.a * ts2.a + ts1.c * ts2.b,
            b: ts1.b * ts2.a + ts1.d * ts2.b,
            c: ts1.a * ts2.c + ts1.c * ts2.d,
            d: ts1.b * ts2.c + ts1.d * ts2.d,
            e: ts1.a * ts2.e + ts1.c * ts2.f + ts1.e,
            f: ts1.b * ts2.e + ts1.d * ts2.f + ts1.f,
        }
    }

    fn apply_to(&self, x: &mut f32, y: &mut f32) {
        let tx = *x;
        let ty = *y;
        *x = self.a * tx + self.c * ty + self.e;
        *y = self.b * tx + self.d * ty + self.f;
    }

    fn is_default(&self) -> bool {
        // A direct float comparison is fine in our case.
           self.a == 1.0
        && self.b == 0.0
        && self.c == 0.0
        && self.d == 1.0
        && self.e == 0.0
        && self.f == 0.0
    }
}

impl Default for Transform {
    #[inline]
    fn default() -> Self {
        Transform { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: 0.0, f: 0.0 }
    }
}

impl std::fmt::Debug for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Transform({} {} {} {} {} {})", self.a, self.b, self.c, self.d, self.e, self.f)
    }
}


#[derive(Clone, Copy, Debug)]
struct GlyphPoint {
    x: i16,
    y: i16,
    on_curve_point: bool,
}


struct GlyphPoints<'a> {
    flags: Stream<'a>,
    x_coords: Stream<'a>,
    y_coords: Stream<'a>,
    points_left: u16,
    flag_repeats: u8,
    last_flags: SimpleGlyphFlags,
    x: i16,
    y: i16,
}

impl<'a> Iterator for GlyphPoints<'a> {
    type Item = GlyphPoint;

    fn next(&mut self) -> Option<Self::Item> {
        type Flags = SimpleGlyphFlags;

        if self.points_left == 0 {
            return None;
        }

        if self.flag_repeats == 0 {
            self.last_flags = Flags::from_bits_truncate(self.flags.read().ok()?);
            if self.last_flags.contains(Flags::REPEAT_FLAG) {
                self.flag_repeats = self.flags.read().ok()?;
            }
        } else {
            self.flag_repeats -= 1;
        }

        let x = get_glyph_coord(
            self.last_flags,
            Flags::X_SHORT_VECTOR,
            Flags::X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR,
            &mut self.x_coords,
        ).ok()?;
        self.x = self.x.wrapping_add(x);

        let y = get_glyph_coord(
            self.last_flags,
            Flags::Y_SHORT_VECTOR,
            Flags::Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR,
            &mut self.y_coords,
        ).ok()?;
        self.y = self.y.wrapping_add(y);

        self.points_left -= 1;

        Some(GlyphPoint {
            x: self.x,
            y: self.y,
            on_curve_point: self.last_flags.contains(Flags::ON_CURVE_POINT),
        })
    }
}

fn get_glyph_coord(
    flags: SimpleGlyphFlags,
    short_vector: SimpleGlyphFlags,
    is_same_or_positive_short_vector: SimpleGlyphFlags,
    coords: &mut Stream,
) -> Result<i16> {
    let flags = (
        flags.contains(short_vector),
        flags.contains(is_same_or_positive_short_vector),
    );

    Ok(match flags {
        (true, true) => {
            coords.read::<u8>()? as i16
        }
        (true, false) => {
            -(coords.read::<u8>()? as i16)
        }
        (false, true) => {
            // Keep previous coordinate.
            0
        }
        (false, false) => {
            coords.read()?
        }
    })
}


#[derive(Clone, Copy, Debug)]
struct Point {
    x: f32,
    y: f32,
}

impl Point {
    #[inline]
    fn lerp(&self, other: Point, t: f32) -> Point {
        Point {
            x: self.x + t * (other.x - self.x),
            y: self.y + t * (other.y - self.y),
        }
    }
}
