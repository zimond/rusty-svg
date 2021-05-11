const fs = require('fs');
const {RustySvg} = require('../pkg/index');

const svg = fs.readFileSync(__dirname + '/duck.svg');
const svgStr = svg.toString();
const rusty = new RustySvg(svgStr);
rusty.cubic_path_to_quad(0.5);
console.log(rusty.to_string())
const buffer = rusty.render();
console.log(buffer.length);