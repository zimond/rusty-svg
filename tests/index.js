const fs = require('fs');
const {RustySvg} = require('../pkg/index');

const svg = fs.readFileSync(__dirname + '/duck.svg');
const svgStr = svg.toString();
const rusty = new RustySvg(svgStr);
const buffer = rusty.render();
console.log(buffer.length);