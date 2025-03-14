pub mod indx;
pub mod meta;
pub mod xmet;
pub mod body;
pub mod foot;

use std::io::Write;

pub use indx::Indx;
pub use meta::Meta;
pub use xmet::Xmet;
pub use body::Body;
pub use foot::Foot;

use crate::{error::WriteError, Head};

pub trait ChunkWrite {
    const FOURCC: [u8; 4];

    fn write(&self, head: &Head, writer: &mut impl Write) -> Result<(), WriteError>;
}
