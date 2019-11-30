use objc::rc::{StrongPtr};

///
/// Represents a canvas that is drawn on using Metal (via gfx) rather than the Quartz drawing operations
///
pub struct MetalCanvas {
    /// The MTLDevice that is implementing this canvas
    device: StrongPtr
}

impl MetalCanvas {
    ///
    /// Creates a new metal canvas
    ///
    pub fn new(device: StrongPtr) -> MetalCanvas {
        MetalCanvas {
            device: device
        }
    }

    ///
    /// Draws a test pattern using the specified drawable
    ///
    pub fn draw_test_pattern(&mut self, drawable: StrongPtr) {
        println!("Test pattern");
    }
}
