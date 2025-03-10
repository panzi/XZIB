use crate::{Head, XZIB};

#[derive(Debug)]
pub struct Builder {
    image: XZIB
}

impl Builder {
    #[inline]
    pub fn new(width: u32, height: u32, channels: u8) -> Self {
        Self {
            image: XZIB {
                head: Head {
                    flags: 0,
                    channels,
                    width,
                    height,
                    planes: 0,
                    index_planes: 0,
                },
                indx: None,
                meta: None,
                xmet: None,
                body: None,
                foot: None,
            }
        }
    }

    // TODO
}
