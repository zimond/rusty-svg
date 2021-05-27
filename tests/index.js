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
const phoneSvg = fs.readFileSync(__dirname + '/tv.svg');
const rusty2 = new RustySvg(phoneSvg.toString());
let bbox = rusty2.inner_bbox();
console.log('bbox width: ', bbox.width);
console.log('bbox height: ', bbox.height);
rusty2.crop(bbox);
fs.writeFileSync(__dirname + '/tv.png', rusty2.render(500/bbox.height));
