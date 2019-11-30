use objc::rc::{StrongPtr};
use gfx_backend_metal as backend;
use gfx_hal::{Instance};
use gfx_hal::adapter;

///
/// Represents a canvas that is drawn on using Metal (via gfx) rather than the Quartz drawing operations
///
pub struct MetalCanvas {
    backend: backend::Instance,
    surface: backend::Surface,
    adapter: adapter::Adapter<backend::Backend>
}

impl MetalCanvas {
    ///
    /// Creates a new metal canvas
    ///
    pub fn new(layer: StrongPtr) -> MetalCanvas {
        unsafe {
            // Create an instance of the Metal back-end 
            // The parameters appear to be ignored and are both unnamed and undocumented in the metal gfx backend. I think they're
            // 'name' and 'version' maybe?
            let metal_backend   = backend::Instance::create("flo_cocoa_ui", 1).expect("Failed to initialise Metal backend");

            // We need an adapter for... something (it's where the queues come from).
            // It's not clear what multiple adapters could mean, the examples all just use the first one like this
            let adapter         = metal_backend.enumerate_adapters().remove(0);

            // Create a surface from our layer
            let surface         = metal_backend.create_surface_from_layer(*layer, false);

            MetalCanvas {
                backend: metal_backend,
                surface: surface,
                adapter: adapter
            }
        }
    }

    ///
    /// Draws a test pattern using the specified drawable
    ///
    pub fn draw_test_pattern(&mut self, drawable: StrongPtr) {
        println!("Test pattern");
    }
}
