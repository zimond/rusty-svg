use js_sys::Uint8Array;
use lyon_algorithms::geom::Point;
use pathfinder_content::stroke::{OutlineStrokeToFill, StrokeStyle};
use pathfinder_content::{
    outline::{Contour, Outline},
    stroke::LineJoin,
};
use pathfinder_geometry::vector::Vector2F;
use std::rc::Rc;
use tiny_skia::Pixmap;
use usvg::{PathData, Transform, NodeExt};
use wasm_bindgen::prelude::*;
extern crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

const MAX: f64 = std::f64::MAX;

#[wasm_bindgen]
#[derive(Debug)]
pub struct BBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[wasm_bindgen(typescript_custom_section)]
const ICONFIG: &str = r#"
interface IConfig {
    width?: number;
    background?: string;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "IConfig")]
    pub type IConfig;
    #[wasm_bindgen(getter, method)]
    pub fn width(this: &IConfig) -> Option<f64>;
    #[wasm_bindgen(getter, method)]
    pub fn background(this: &IConfig) -> Option<String>;
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
        let opt = usvg::Options::default();
        let tree = usvg::Tree::from_str(svg, &opt.to_ref()).unwrap();
        let mut svg = RustySvg { tree };
        svg.apply_transform();
        svg
    }

    #[wasm_bindgen(getter)]
    pub fn width(&self) -> f64 {
        self.tree.svg_node().size.width()
    }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> f64 {
        self.tree.svg_node().size.height()
    }

    /// Render the svg to PNG buffer. Accepts an optional `width`, allowing
    /// the image to be scaled proportionally based on the given width.
    ///
    /// Note: floated width will be floored to integer value
    pub fn render(&self, config: Option<IConfig>) -> Option<Uint8Array> {
        let width = config
            .as_ref()
            .and_then(|conf| conf.width())
            .unwrap_or(self.width());
        let height = width / self.width() * self.height();
        let background = config
            .as_ref()
            .and_then(|conf| conf.background())
            .and_then(|color| color.parse::<usvg::Color>().ok());
        let mut pixmap = Pixmap::new(width as u32, height as u32)?;
        if let Some(color) = background {
            pixmap.fill(tiny_skia::Color::from_rgba8(
                color.red,
                color.green,
                color.blue,
                255,
            ));
        }
        resvg::render(
            &self.tree,
            usvg::FitTo::Width(width as u32),
            pixmap.as_mut(),
        )?;
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
                            let seg = lyon_algorithms::geom::CubicBezierSegment {
                                from: Point::new(from.0, from.1),
                                ctrl1: Point::new(*x1, *y1),
                                ctrl2: Point::new(*x2, *y2),
                                to: Point::new(*x, *y),
                            };
                            lyon_algorithms::geom::cubic_to_quadratic::cubic_to_quadratics(
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

    /// Output the svg to a string
    pub fn to_string(&self) -> String {
        let s = self.tree.to_string(&usvg::XmlOptions::default());
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

    /// Calculate a maximum bounding box of all visible elements in this
    /// SVG.
    ///
    /// Note: path bounding box are approx. values
    pub fn inner_bbox(&self) -> BBox {
        let mut min_point = usvg::Point::new(MAX, MAX);
        let mut max_point = usvg::Point::new(-MAX, -MAX);
        for child in self.tree.root().children().skip(1) {
            let (min, max) = self.node_bbox(child);
            min_max_point(&mut min_point, &mut max_point, min.x, min.y);
            min_max_point(&mut min_point, &mut max_point, max.x, max.y);
        }
        let rect = self.tree.svg_node().view_box.rect;
        min_point.x = min_point.x.max(rect.x());
        min_point.y = min_point.y.max(rect.y());
        max_point.x = max_point.x.min(rect.x() + rect.width());
        max_point.y = max_point.y.min(rect.y() + rect.height());
        BBox {
            x: min_point.x.floor(),
            y: min_point.y.floor(),
            width: max_point.x.ceil() - min_point.x.floor(),
            height: max_point.y.ceil() - min_point.y.floor(),
        }
    }

    /// Calculate SVG bbox, like SVGGraphicsElement.getBBox() DOM API.
    pub fn get_bbox(&self) -> BBox {
        let node = self.tree.root();
        if let Some(bbox) = node.calculate_bbox() {
            return BBox {
                x: bbox.x(),
                y: bbox.y(),
                width: bbox.width(),
                height: bbox.height(),
            }
        } else {
            log!("SVG {:?} has zero size.", node.id());
            return BBox {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            }
        }
    }

    /// Use a given `BBox` to crop the svg. Currently this method simply
    /// changes the viewbox/size of the svg and do not move the elements
    /// for simplicity
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

    // Currently this method only applies transforms added to paths
    fn apply_transform(&mut self) {
        for mut node in self.tree.root().descendants() {
            if let usvg::NodeKind::Path(p) = &mut *node.borrow_mut() {
                let transform = p.transform;
                if transform.is_default() {
                    continue;
                }
                let mut data = p.data.0.clone();
                for seg in &mut data {
                    match seg {
                        usvg::PathSegment::MoveTo { x, y } => {
                            transform.apply_to(x, y);
                        }
                        usvg::PathSegment::LineTo { x, y } => {
                            transform.apply_to(x, y);
                        }
                        usvg::PathSegment::CurveTo {
                            x1,
                            x2,
                            y1,
                            y2,
                            x,
                            y,
                        } => {
                            transform.apply_to(x, y);
                            transform.apply_to(x1, y1);
                            transform.apply_to(x2, y2);
                        }
                        _ => {}
                    }
                }
                p.data = Rc::new(PathData(data));
                p.transform = Transform::default();
            }
        }
    }

    fn node_bbox(&self, node: usvg::Node) -> (usvg::Point<f64>, usvg::Point<f64>) {
        let transform = node.borrow().transform();
        let (bbox_min, bbox_max) = match &*node.borrow() {
            usvg::NodeKind::Path(p) => {
                let mut outline = Outline::new();
                let mut contour = Contour::new();
                let mut iter = p.data.0.iter().peekable();
                while let Some(seg) = iter.next() {
                    match seg {
                        usvg::PathSegment::MoveTo { x, y } => {
                            if !contour.is_empty() {
                                outline
                                    .push_contour(std::mem::replace(&mut contour, Contour::new()));
                            }
                            contour.push_endpoint(Vector2F::new(*x as f32, *y as f32));
                        }
                        usvg::PathSegment::LineTo { x, y } => {
                            let v = Vector2F::new(*x as f32, *y as f32);
                            if let Some(usvg::PathSegment::ClosePath) = iter.peek() {
                                let first = contour.position_of(0);
                                if (first - v).square_length() < 1.0 {
                                    continue;
                                }
                            }
                            contour.push_endpoint(v);
                        }
                        usvg::PathSegment::CurveTo {
                            x1,
                            y1,
                            x2,
                            y2,
                            x,
                            y,
                        } => {
                            contour.push_cubic(
                                Vector2F::new(*x1 as f32, *y1 as f32),
                                Vector2F::new(*x2 as f32, *y2 as f32),
                                Vector2F::new(*x as f32, *y as f32),
                            );
                        }
                        usvg::PathSegment::ClosePath => {
                            contour.close();
                            outline.push_contour(std::mem::replace(&mut contour, Contour::new()));
                        }
                    }
                }
                if !contour.is_empty() {
                    outline.push_contour(std::mem::replace(&mut contour, Contour::new()));
                }
                if let Some(stroke) = p.stroke.as_ref() {
                    let mut style = StrokeStyle::default();
                    style.line_width = stroke.width.value() as f32;
                    style.line_join = LineJoin::Miter(style.line_width);
                    let mut filler = OutlineStrokeToFill::new(&outline, style);
                    filler.offset();
                    outline = filler.into_outline();
                }
                let bbox = outline.bounds();
                (
                    usvg::Point::new(bbox.origin_x() as f64, bbox.origin_y() as f64),
                    usvg::Point::new(bbox.lower_right().x() as f64, bbox.lower_right().y() as f64),
                )
            }
            usvg::NodeKind::Group(g) => {
                if let Some(clippath) = g.clip_path.as_ref().and_then(|cp| self.node_by_id(cp)) {
                    self.node_bbox(clippath)
                } else if let Some(mask) = g.mask.as_ref().and_then(|cp| self.node_by_id(cp)) {
                    self.node_bbox(mask)
                } else {
                    let mut min_point = usvg::Point::new(MAX, MAX);
                    let mut max_point = usvg::Point::new(-MAX, -MAX);
                    for child in node.children() {
                        let (child_min, child_max) = self.node_bbox(child);
                        min_max_point(&mut min_point, &mut max_point, child_min.x, child_min.y);
                        min_max_point(&mut min_point, &mut max_point, child_max.x, child_max.y);
                    }
                    (min_point, max_point)
                }
            }
            usvg::NodeKind::Image(image) => {
                let rect = image.view_box.rect;
                (
                    usvg::Point::new(rect.x(), rect.y()),
                    usvg::Point::new(rect.right(), rect.bottom()),
                )
            }
            usvg::NodeKind::ClipPath(_) | usvg::NodeKind::Mask(_) => {
                if let Some(child) = node.first_child() {
                    return self.node_bbox(child);
                } else {
                    let min_point = usvg::Point::new(MAX, MAX);
                    let max_point = usvg::Point::new(-MAX, -MAX);
                    return (min_point, max_point);
                }
            }
            _ => {
                let min_point = usvg::Point::new(MAX, MAX);
                let max_point = usvg::Point::new(-MAX, -MAX);
                return (min_point, max_point);
            }
        };
        let (x1, y1) = transform.apply(bbox_min.x, bbox_min.y);
        let (x2, y2) = transform.apply(bbox_max.x, bbox_max.y);
        let (x3, y3) = transform.apply(bbox_min.x, bbox_max.y);
        let (x4, y4) = transform.apply(bbox_max.x, bbox_min.y);
        let x_min = x1.min(x2).min(x3).min(x4);
        let x_max = x1.max(x2).max(x3).max(x4);
        let y_min = y1.min(y2).min(y3).min(y4);
        let y_max = y1.max(y2).max(y3).max(y4);
        (
            usvg::Point::new(x_min, y_min),
            usvg::Point::new(x_max, y_max),
        )
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

fn min_max_point(
    min_point: &mut usvg::Point<f64>,
    max_point: &mut usvg::Point<f64>,
    x: f64,
    y: f64,
) {
    if x != -MAX {
        min_point.x = min_point.x.min(x);
    }
    if y != -MAX {
        min_point.y = min_point.y.min(y);
    }
    if x != MAX {
        max_point.x = max_point.x.max(x);
    }
    if y != MAX {
        max_point.y = max_point.y.max(y);
    }
}

#[cfg(test)]
mod test {
    use super::RustySvg;
    use std::fs::File;
    use std::io::Read;
    #[test]
    fn test_inner_bbox() {
        let mut file = File::open("tests/heart.svg").unwrap();
        let mut svg = String::new();
        file.read_to_string(&mut svg).unwrap();
        let svg = RustySvg::new(&svg);
        assert_eq!(svg.inner_bbox().width.round() as u32, 116);
        // TODO: test inner_bbox().height
        // assert_eq!(svg.inner_bbox().height, 87.28472137451172);
    }
    #[test]
    fn test_get_bbox() {
        let mut file = File::open("tests/heart.svg").unwrap();
        let mut svg = String::new();
        file.read_to_string(&mut svg).unwrap();
        let svg = RustySvg::new(&svg);
        assert_eq!(svg.get_bbox().width as f64, 119.30921936035156);
        assert_eq!(svg.get_bbox().height as f64, 87.28472137451172);
    }
}
