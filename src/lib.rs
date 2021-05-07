use js_sys::Uint8Array;
use tiny_skia::Pixmap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct RustySvg {
    tree: usvg::Tree,
}

fn main() {}

#[wasm_bindgen]
impl RustySvg {
    #[wasm_bindgen(constructor)]
    pub fn new(svg: &str) -> RustySvg {
        let tree = usvg::Tree::from_str(svg, &usvg::Options::default()).unwrap();
        RustySvg { tree }
    }

    #[wasm_bindgen(getter)]
    pub fn width(&self) -> f64 {
        self.tree.svg_node().size.width()
    }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> f64 {
        self.tree.svg_node().size.height()
    }

    pub fn render(&self) -> Option<Uint8Array> {
        let mut pixmap = Pixmap::new(self.width() as u32, self.height() as u32)?;
        resvg::render(&self.tree, usvg::FitTo::Original, pixmap.as_mut())?;
        let buffer = pixmap.encode_png().unwrap();
        Some(Uint8Array::from(buffer.as_slice()))
    }
}
