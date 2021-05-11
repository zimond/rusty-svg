use js_sys::Uint8Array;
use lyon_geom::Point;
use std::rc::Rc;
use tiny_skia::Pixmap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct RustySvg {
    tree: usvg::Tree,
}

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

    pub fn render(&self, factor: Option<f64>) -> Option<Uint8Array> {
        let ratio = factor.unwrap_or(1.0);
        let mut pixmap = Pixmap::new(
            (self.width() * ratio) as u32,
            (self.height() * ratio) as u32,
        )?;
        resvg::render(&self.tree, usvg::FitTo::Zoom(ratio as f32), pixmap.as_mut())?;
        let buffer = pixmap.encode_png().unwrap();
        Some(Uint8Array::from(buffer.as_slice()))
    }

    /// Resolve all cubic path segments in this SVG to quadratic segments
    pub fn cubic_path_to_quad(&self, tolerance: f64) {
        for mut node in self.tree.root().descendants() {
            if let usvg::NodeKind::Path(p) = &mut *node.borrow_mut() {
                let mut new_data = vec![];
                let mut from = (0.0, 0.0);
                let mut start = (0.0, 0.0);
                for seg in &p.data.0 {
                    match seg {
                        usvg::PathSegment::CurveTo {
                            x1,
                            y1,
                            x2,
                            y2,
                            x,
                            y,
                        } => {
                            let seg = lyon_geom::CubicBezierSegment {
                                from: Point::new(from.0, from.1),
                                ctrl1: Point::new(*x1, *y1),
                                ctrl2: Point::new(*x2, *y2),
                                to: Point::new(*x, *y),
                            };
                            lyon_geom::cubic_to_quadratic::cubic_to_quadratics(
                                &seg,
                                tolerance,
                                &mut |new_seg| {
                                    new_data.push(usvg::PathSegment::CurveTo {
                                        x1: new_seg.ctrl.x,
                                        y1: new_seg.ctrl.y,
                                        x2: new_seg.ctrl.x,
                                        y2: new_seg.ctrl.y,
                                        x: new_seg.to.x,
                                        y: new_seg.to.y,
                                    });
                                },
                            );
                            from = (*x, *y);
                        }
                        usvg::PathSegment::MoveTo { x, y } => {
                            from = (*x, *y);
                            start = (*x, *y);
                            new_data.push(seg.clone());
                        }
                        usvg::PathSegment::LineTo { x, y } => {
                            from = (*x, *y);
                            new_data.push(seg.clone());
                        }
                        usvg::PathSegment::ClosePath => {
                            new_data.push(seg.clone());
                            from = start;
                        }
                    }
                }
                p.data = Rc::new(usvg::PathData(new_data));
            }
        }
    }

    pub fn to_string(&self) -> String {
        let s = self.tree.to_string(usvg::XmlOptions::default());
        let path_reg = regex::RegexBuilder::new(
            r#"\s(C\s[\d\.]+\s[\d\.]+\s[\d\.]+\s[\d\.]+\s[\d\.]+\s[\d\.]+)"#,
        )
        .case_insensitive(true)
        .build()
        .unwrap();
        path_reg
            .replace_all(&s, |d: &regex::Captures| {
                if let Some(cap) = d.get(1) {
                    let mut data = cap.as_str().trim().split(' ').collect::<Vec<_>>();
                    assert_eq!(data.len(), 7);
                    if data[1] == data[3] && data[2] == data[4] {
                        data.remove(4);
                        data.remove(3);
                    }
                    data.remove(0);
                    format!(
                        " Q {}",
                        data.into_iter()
                            .map(|d| d.to_string())
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                } else {
                    d.get(0).unwrap().as_str().to_string()
                }
            })
            .to_string()
    }
}
