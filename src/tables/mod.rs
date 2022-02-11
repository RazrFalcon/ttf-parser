pub mod cbdt;
pub mod cblc;
mod cff;
pub mod cmap;
pub mod glyf;
pub mod head;
pub mod hhea;
pub mod hmtx;
pub mod kern;
pub mod loca;
pub mod maxp;
pub mod name;
pub mod os2;
pub mod post;
pub mod sbix;
pub mod svg;
pub mod vhea;
pub mod vorg;

#[cfg(feature = "opentype-layout")] pub mod gdef;
#[cfg(feature = "opentype-layout")] pub mod gsub;
#[cfg(feature = "opentype-layout")] pub mod gpos;

#[cfg(feature = "opentype-math")] pub mod math;

#[cfg(feature = "apple-layout")] pub mod ankr;
#[cfg(feature = "apple-layout")] pub mod feat;
#[cfg(feature = "apple-layout")] pub mod trak;

#[cfg(feature = "variable-fonts")] pub mod avar;
#[cfg(feature = "variable-fonts")] pub mod fvar;
#[cfg(feature = "variable-fonts")] pub mod gvar;
#[cfg(feature = "variable-fonts")] pub mod hvar;
#[cfg(feature = "variable-fonts")] pub mod mvar;

pub use cff::cff1;
pub use cff::CFFError;
#[cfg(feature = "variable-fonts")] pub use cff::cff2;
