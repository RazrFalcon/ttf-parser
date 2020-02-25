// https://docs.microsoft.com/en-us/typography/opentype/spec/glyf

// This module is a heavily modified version of https://github.com/raphlinus/font-rs

use core::num::NonZeroU16;

use crate::parser::{Stream, F2DOT14, FromData};
use crate::{Font, GlyphId, OutlineBuilder, Rect};

/// A wrapper that transforms segments before passing them to `OutlineBuilder`.
trait OutlineBuilderInner {
    fn move_to(&mut self, x: f32, y: f32);
    fn line_to(&mut self, x: f32, y: f32);
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32);
    fn close(&mut self);
}

struct Builder<'a> {
    builder: &'a mut dyn OutlineBuilder,
    transform: Transform,
    is_default_ts: bool, // `bool` is faster than `Option` or `is_default`.
}

impl<'a> OutlineBuilderInner for Builder<'a> {
    #[inline]
    fn move_to(&mut self, mut x: f32, mut y: f32) {
        if !self.is_default_ts {
            self.transform.apply_to(&mut x, &mut y);
        }

        self.builder.move_to(x, y);
    }

    #[inline]
    fn line_to(&mut self, mut x: f32, mut y: f32) {
        if !self.is_default_ts {
            self.transform.apply_to(&mut x, &mut y);
        }

        self.builder.line_to(x, y);
    }

    #[inline]
    fn quad_to(&mut self, mut x1: f32, mut y1: f32, mut x: f32, mut y: f32) {
        if !self.is_default_ts {
            self.transform.apply_to(&mut x1, &mut y1);
            self.transform.apply_to(&mut x, &mut y);
        }

        self.builder.quad_to(x1, y1, x, y);
    }

    #[inline]
    fn close(&mut self) {
        self.builder.close();
    }
}


// https://docs.microsoft.com/en-us/typography/opentype/spec/glyf#simple-glyph-description
#[derive(Clone, Copy)]
struct SimpleGlyphFlags(u8);

impl SimpleGlyphFlags {
    #[inline] fn on_curve_point(&self) -> bool { self.0 & 0x01 != 0 }
    #[inline] fn x_short(&self) -> bool { self.0 & 0x02 != 0 }
    #[inline] fn y_short(&self) -> bool { self.0 & 0x04 != 0 }
    #[inline] fn repeat_flag(&self) -> bool { self.0 & 0x08 != 0 }
    #[inline] fn x_is_same_or_positive_short(&self) -> bool { self.0 & 0x10 != 0 }
    #[inline] fn y_is_same_or_positive_short(&self) -> bool { self.0 & 0x20 != 0 }
}

impl FromData for SimpleGlyphFlags {
    #[inline]
    fn parse(data: &[u8]) -> Self {
        SimpleGlyphFlags(data[0])
    }
}


// https://docs.microsoft.com/en-us/typography/opentype/spec/glyf#composite-glyph-description
#[derive(Clone, Copy)]
struct CompositeGlyphFlags(u16);

impl CompositeGlyphFlags {
    #[inline] fn arg_1_and_2_are_words(&self) -> bool { self.0 & 0x0001 != 0 }
    #[inline] fn args_are_xy_values(&self) -> bool { self.0 & 0x0002 != 0 }
    #[inline] fn we_have_a_scale(&self) -> bool { self.0 & 0x0008 != 0 }
    #[inline] fn more_components(&self) -> bool { self.0 & 0x0020 != 0 }
    #[inline] fn we_have_an_x_and_y_scale(&self) -> bool { self.0 & 0x0040 != 0 }
    #[inline] fn we_have_a_two_by_two(&self) -> bool { self.0 & 0x0080 != 0 }
}

impl FromData for CompositeGlyphFlags {
    #[inline]
    fn parse(data: &[u8]) -> Self {
        CompositeGlyphFlags(u16::parse(data))
    }
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
        builder: &mut dyn OutlineBuilder,
    ) -> Option<Rect> {
        let mut b = Builder {
            builder,
            transform: Transform::default(),
            is_default_ts: true,
        };

        let glyph_data = self.glyph_data(glyph_id)?;
        self.outline_impl(glyph_data, 0, &mut b)
    }

    pub(crate) fn glyf_glyph_bbox(&self, glyph_id: GlyphId) -> Option<Rect> {
        let glyph_data = self.glyph_data(glyph_id)?;
        let mut s = Stream::new(glyph_data);
        s.skip::<i16>(); // number_of_contours
        // It's faster to parse the rect directly, instead of using `FromData`.
        Some(Rect {
            x_min: s.read()?,
            y_min: s.read()?,
            x_max: s.read()?,
            y_max: s.read()?,
        })
    }

    fn glyph_data(&self, glyph_id: GlyphId) -> Option<&[u8]> {
        let range = self.glyph_range(glyph_id)?;
        let data = self.glyf?;
        data.get(range)
    }

    fn outline_impl(
        &self,
        data: &[u8],
        depth: u8,
        builder: &mut Builder,
    ) -> Option<Rect> {
        if depth >= MAX_COMPONENTS {
            warn!("Recursion detected in the 'glyf' table.");
            return None;
        }

        let mut s = Stream::new(data);
        let number_of_contours: i16 = s.read()?;
        // It's faster to parse the rect directly, instead of using `FromData`.
        let rect = Rect {
            x_min: s.read()?,
            y_min: s.read()?,
            x_max: s.read()?,
            y_max: s.read()?,
        };

        if number_of_contours > 0 {
            let number_of_contours = NonZeroU16::new(number_of_contours as u16)?;
            Self::parse_simple_outline(s.tail()?, number_of_contours, builder)?;
        } else if number_of_contours < 0 {
            self.parse_composite_outline(s.tail()?, depth + 1, builder)?;
        } else {
            // An empty glyph.
            return None;
        }

        Some(rect)
    }

    #[inline(never)]
    fn parse_simple_outline(
        glyph_data: &[u8],
        number_of_contours: NonZeroU16,
        builder: &mut Builder,
    ) -> Option<()> {
        let mut s = Stream::new(glyph_data);
        let endpoints = s.read_array::<u16, u16>(number_of_contours.get())?;

        let points_total = {
            let last_point = endpoints.last()?;
            // Prevent overflow.
            last_point.checked_add(1)?
        };

        // Skip instructions byte code.
        let instructions_len: u16 = s.read()?;
        s.advance(instructions_len);

        let flags_offset = s.offset();
        let x_coords_len = Self::resolve_x_coords_len(&mut s, points_total)?;
        let x_coords_offset = s.offset();
        let y_coords_offset = x_coords_offset + x_coords_len as usize;

        let mut points = GlyphPoints {
            flags: Stream::new(glyph_data.get(flags_offset..x_coords_offset)?),
            x_coords: Stream::new(glyph_data.get(x_coords_offset..y_coords_offset)?),
            y_coords: Stream::new(glyph_data.get(y_coords_offset..glyph_data.len())?),
            points_left: points_total,
            flag_repeats: 0,
            last_flags: SimpleGlyphFlags(0),
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
            if n == core::u16::MAX {
                break;
            }

            let n = n + 1 - total;

            // Contour must have at least 2 points.
            if n >= 2 {
                Self::points_to_contour(points.by_ref().take(n as usize), builder);
            }

            total += n;
        }

        Some(())
    }

    /// Resolves the X coordinates length.
    ///
    /// The length depends on *Simple Glyph Flags*, so we have to process them all to find it.
    fn resolve_x_coords_len(
        s: &mut Stream,
        points_total: u16,
    ) -> Option<u16> {
        let mut flags_left = points_total;
        let mut x_coords_len = 0u16;
        while flags_left > 0 {
            let flags: SimpleGlyphFlags = s.read()?;

            // The number of times a glyph point repeats.
            let repeats = if flags.repeat_flag() {
                let repeats: u8 = s.read()?;
                repeats as u16 + 1
            } else {
                1
            };

            if flags.x_short() {
                // Coordinate is 1 byte long.
                x_coords_len = x_coords_len.checked_add(repeats)?;
            } else if !flags.x_is_same_or_positive_short() {
                // Coordinate is 2 bytes long.
                x_coords_len = x_coords_len.checked_add(repeats * 2)?;
            }

            // Check for overflow.
            // Do not use checked_sub, because it's very slow for some reasons.
            if repeats <= flags_left {
                flags_left -= repeats;
            } else {
                return None;
            }
        }

        Some(x_coords_len)
    }

    /// Useful links:
    ///
    /// - https://developer.apple.com/fonts/TrueType-Reference-Manual/RM01/Chap1.html
    /// - https://stackoverflow.com/a/20772557
    fn points_to_contour(
        points: core::iter::Take<&mut GlyphPoints>,
        builder: &mut Builder,
    ) {
        let mut first_oncurve: Option<Point> = None;
        let mut first_offcurve: Option<Point> = None;
        let mut last_offcurve: Option<Point> = None;
        for point in points {
            let p = Point { x: point.x as f32, y: point.y as f32 };
            if first_oncurve.is_none() {
                if point.on_curve_point {
                    first_oncurve = Some(p);
                    builder.move_to(p.x, p.y);
                } else {
                    if let Some(offcurve) = first_offcurve {
                        let mid = offcurve.lerp(p, 0.5);
                        first_oncurve = Some(mid);
                        last_offcurve = Some(p);
                        builder.move_to(mid.x, mid.y);
                    } else {
                        first_offcurve = Some(p);
                    }
                }
            } else {
                match (last_offcurve, point.on_curve_point) {
                    (Some(offcurve), true) => {
                        last_offcurve = None;
                        builder.quad_to(offcurve.x, offcurve.y, p.x, p.y);
                    }
                    (Some(offcurve), false) => {
                        last_offcurve = Some(p);
                        let mid = offcurve.lerp(p, 0.5);
                        builder.quad_to(offcurve.x, offcurve.y, mid.x, mid.y);
                    }
                    (None, true) => {
                        builder.line_to(p.x, p.y);
                    }
                    (None, false) => {
                        last_offcurve = Some(p);
                    }
                }
            }
        }

        loop {
            if let (Some(offcurve1), Some(offcurve2)) = (first_offcurve, last_offcurve) {
                last_offcurve = None;
                let mid = offcurve2.lerp(offcurve1, 0.5);
                builder.quad_to(offcurve2.x, offcurve2.y, mid.x, mid.y);
            } else {
                if let (Some(p), Some(offcurve1)) = (first_oncurve, first_offcurve) {
                    builder.quad_to(offcurve1.x, offcurve1.y, p.x, p.y);
                } else if let (Some(p), Some(offcurve2)) = (first_oncurve, last_offcurve) {
                    builder.quad_to(offcurve2.x, offcurve2.y, p.x, p.y);
                } else if let Some(p) = first_oncurve {
                    builder.line_to(p.x, p.y);
                }

                break;
            }
        }

        builder.close();
    }

    #[inline(never)]
    fn parse_composite_outline(
        &self,
        glyph_data: &[u8],
        depth: u8,
        builder: &mut Builder,
    ) -> Option<()> {
        if depth >= MAX_COMPONENTS {
            warn!("Recursion detected in the 'glyf' table.");
            return None;
        }

        let mut s = Stream::new(glyph_data);
        let flags: CompositeGlyphFlags = s.read()?;
        let glyph_id: GlyphId = s.read()?;

        let mut ts = Transform::default();

        if flags.args_are_xy_values() {
            if flags.arg_1_and_2_are_words() {
                ts.e = s.read::<i16>()? as f32;
                ts.f = s.read::<i16>()? as f32;
            } else {
                ts.e = s.read::<i8>()? as f32;
                ts.f = s.read::<i8>()? as f32;
            }
        }

        if flags.we_have_a_two_by_two() {
            ts.a = s.read::<F2DOT14>()?.0;
            ts.b = s.read::<F2DOT14>()?.0;
            ts.c = s.read::<F2DOT14>()?.0;
            ts.d = s.read::<F2DOT14>()?.0;
        } else if flags.we_have_an_x_and_y_scale() {
            ts.a = s.read::<F2DOT14>()?.0;
            ts.d = s.read::<F2DOT14>()?.0;
        } else if flags.we_have_a_scale() {
            // 'If the bit WE_HAVE_A_SCALE is set, the scale value is read in 2.14 format.
            // The value can be between -2 to almost +2.'
            ts.a = f32_bound(-2.0, s.read::<F2DOT14>()?.0, 2.0);
            ts.d = ts.a;
        }

        if let Some(glyph_data) = self.glyph_data(glyph_id) {
            let transform = Transform::combine(builder.transform, ts);
            let mut b = Builder {
                builder: builder.builder,
                transform,
                is_default_ts: transform.is_default(),
            };

            self.outline_impl(glyph_data, depth + 1, &mut b)?;
        }

        if flags.more_components() {
            if depth < MAX_COMPONENTS {
                self.parse_composite_outline(s.tail()?, depth + 1, builder)?;
            }
        }

        Some(())
    }
}


#[derive(Clone, Copy)]
struct Transform {
    a: f32, b: f32, c: f32,
    d: f32, e: f32, f: f32,
}

impl Transform {
    #[inline]
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

    #[inline]
    fn apply_to(&self, x: &mut f32, y: &mut f32) {
        let tx = *x;
        let ty = *y;
        *x = self.a * tx + self.c * ty + self.e;
        *y = self.b * tx + self.d * ty + self.f;
    }

    #[inline]
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

impl core::fmt::Debug for Transform {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Transform({} {} {} {} {} {})", self.a, self.b, self.c, self.d, self.e, self.f)
    }
}


#[derive(Clone, Copy, Debug)]
struct GlyphPoint {
    x: i16,
    y: i16,
    /// Indicates that a point is a point on curve
    /// and not a control point.
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
        if self.points_left == 0 {
            return None;
        }

        if self.flag_repeats == 0 {
            self.last_flags = self.flags.read()?;
            if self.last_flags.repeat_flag() {
                self.flag_repeats = self.flags.read()?;
            }
        } else {
            self.flag_repeats -= 1;
        }

        let x = match (self.last_flags.x_short(), self.last_flags.x_is_same_or_positive_short()) {
            (true, true) => {
                self.x_coords.read::<u8>()? as i16
            }
            (true, false) => {
                -(self.x_coords.read::<u8>()? as i16)
            }
            (false, true) => {
                // Keep previous coordinate.
                0
            }
            (false, false) => {
                self.x_coords.read()?
            }
        };
        self.x = self.x.wrapping_add(x);

        let y = match (self.last_flags.y_short(), self.last_flags.y_is_same_or_positive_short()) {
            (true, true) => {
                self.y_coords.read::<u8>()? as i16
            }
            (true, false) => {
                -(self.y_coords.read::<u8>()? as i16)
            }
            (false, true) => {
                // Keep previous coordinate.
                0
            }
            (false, false) => {
                self.y_coords.read()?
            }
        };
        self.y = self.y.wrapping_add(y);

        self.points_left -= 1;

        Some(GlyphPoint {
            x: self.x,
            y: self.y,
            on_curve_point: self.last_flags.on_curve_point(),
        })
    }
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
