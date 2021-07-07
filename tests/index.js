const fs = require('fs');
const { RustySvg, BBox } = require('../pkg/index');

const svg = fs.readFileSync(__dirname + '/tiger.svg');
const svgStr = svg.toString();
const rusty = new RustySvg(svgStr);
const pngBuffer = rusty.render();
console.log('pngBuffer length: ', pngBuffer.length);
fs.writeFileSync(__dirname + '/tiger.png', pngBuffer);

rusty.cubic_path_to_quad(0.5);
fs.writeFileSync(__dirname + '/tiger-quadratic.svg', rusty.to_string());

// test BBox
const svg2 = fs.readFileSync(__dirname + '/heart.svg');
const rusty2 = new RustySvg(svg2.toString());
let bbox = rusty2.inner_bbox();
console.log('bbox width: ', bbox.width, '浏览器 119.30921936035156');
console.log('bbox height: ', bbox.height, '浏览器 87.28472137451172');
rusty2.crop(bbox); // 裁剪为 BBox 大小

console.log(rusty2.to_string());
// 指定生成的 png 宽度 500
fs.writeFileSync(__dirname + '/heart.png', rusty2.render(500));

const svg3 = fs.readFileSync(__dirname + '/book.svg');
const rusty3 = new RustySvg(svg3.toString());
let bbox2 = rusty3.inner_bbox();
console.log('bbox width: ', bbox2.width);
console.log('bbox height: ', bbox2.height);
rusty3.crop(bbox2);
fs.writeFileSync(__dirname + '/book.png', rusty3.render());
