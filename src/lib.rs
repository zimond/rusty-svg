use js_sys::Uint8Array;
use lyon_geom::Point;
use std::rc::Rc;
use tiny_skia::Pixmap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct BBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[wasm_bindgen]
impl BBox {
    #[wasm_bindgen(constructor)]
    pub fn new() -> BBox {
        BBox {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        }
    }
}

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
                    let indicator = if data[1] == data[3] && data[2] == data[4] {
                        data.remove(4);
                        data.remove(3);
                        "Q"
                    } else {
                        "C"
                    };
                    data.remove(0);
                    format!(
                        " {} {}",
                        indicator,
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

    pub fn inner_bbox(&self) -> BBox {
        let mut min_point = Point::new(std::f64::MAX, std::f64::MAX);
        let mut max_point = Point::new(0.0, 0.0);
        for child in self.tree.root().children().skip(1) {
            for node in child.descendants() {
                self.node_bbox(node, &mut min_point, &mut max_point);
            }
        }
        BBox {
            x: min_point.x,
            y: min_point.y,
            width: max_point.x - min_point.x,
            height: max_point.y - min_point.y,
        }
    }

    pub fn crop(&mut self, bbox: &BBox) {
        let mut node = self.tree.root();
        let mut node = node.borrow_mut();
        if let usvg::NodeKind::Svg(svg) = &mut *node {
            let ratio_x = svg.size.width() / svg.view_box.rect.width();
            let ratio_y = svg.size.height() / svg.view_box.rect.height();
            svg.view_box.rect = usvg::Rect::new(bbox.x, bbox.y, bbox.width, bbox.height).unwrap();
            svg.size = usvg::Size::new(bbox.width * ratio_x, bbox.height * ratio_y).unwrap();
        }
    }

    fn node_bbox(&self, node: usvg::Node, min_point: &mut Point<f64>, max_point: &mut Point<f64>) {
        match &*node.borrow() {
            usvg::NodeKind::Path(p) => {
                for seg in &p.data.0 {
                    match seg {
                        usvg::PathSegment::MoveTo { x, y } => {
                            min_max_point(min_point, max_point, *x, *y)
                        }
                        usvg::PathSegment::LineTo { x, y } => {
                            min_max_point(min_point, max_point, *x, *y)
                        }
                        usvg::PathSegment::CurveTo {
                            x1,
                            y1,
                            x2,
                            y2,
                            x,
                            y,
                        } => {
                            min_max_point(min_point, max_point, *x1, *y1);
                            min_max_point(min_point, max_point, *x2, *y2);
                            min_max_point(min_point, max_point, *x, *y)
                        }
                        _ => {}
                    }
                }
            }
            usvg::NodeKind::Group(g) => {
                if let Some(clippath) = g.clip_path.as_ref().and_then(|cp| self.node_by_id(cp)) {
                    return self.node_bbox(clippath, min_point, max_point);
                }
                if let Some(mask) = g.mask.as_ref().and_then(|cp| self.node_by_id(cp)) {
                    return self.node_bbox(mask, min_point, max_point);
                }
                for child in node.children() {
                    self.node_bbox(child, min_point, max_point);
                }
            }
            usvg::NodeKind::Image(image) => {
                let (x, y) = image
                    .transform
                    .apply(image.view_box.rect.x(), image.view_box.rect.y());
                let (x2, y2) = image
                    .transform
                    .apply(image.view_box.rect.right(), image.view_box.rect.bottom());
                min_max_point(min_point, max_point, x, y);
                min_max_point(min_point, max_point, x2, y2);
            }
            _ => {}
        }
    }

    fn node_by_id(&self, id: &str) -> Option<usvg::Node> {
        for node in self.tree.root().descendants() {
            if id == node.borrow().id() {
                return Some(node);
            }
        }
        None
    }
}

fn min_max_point(min_point: &mut Point<f64>, max_point: &mut Point<f64>, x: f64, y: f64) {
    min_point.x = min_point.x.min(x);
    min_point.y = min_point.y.min(y);
    max_point.x = max_point.x.max(x);
    max_point.y = max_point.y.max(y);
}
