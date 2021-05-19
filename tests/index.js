const fs = require('fs');
const { RustySvg } = require('../pkg/index');

const svg = fs.readFileSync(__dirname + '/tiger.svg');
const svgStr = svg.toString();
const rusty = new RustySvg(svgStr);
const pngBuffer = rusty.render();
console.log('pngBuffer length: ', pngBuffer.length);
fs.writeFileSync(__dirname + '/tiger.png', pngBuffer);

rusty.cubic_path_to_quad(0.5);
fs.writeFileSync(__dirname + '/tiger-quadratic.svg', rusty.to_string());
