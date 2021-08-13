# rusty-svg

<p>
  <a href="https://npmjs.com/package/rusty-svg"><img src="https://img.shields.io/npm/v/rusty-svg.svg" alt="npm package"></a>
  <a href="https://github.com/zimond/rusty-svg/actions/workflows/ci.yml"><img src="https://github.com/zimond/rusty-svg/actions/workflows/ci.yml/badge.svg?branch=main" alt="build status"></a>
</p>

An SVG toolkit based on [resvg](https://github.com/RazrFalcon/resvg)

This module is compiled to WASM and currently only supports Node.js

Comparing with the backend ReSVG, this module removes text support as it requires complex
font loading logic.

## Example

Run the following command, it will convert [tiger.svg](tests/tiger.svg) to [tiger.png](tests/tiger.png)

```shell
node tests/index.js
```

| SVG                                                     | PNG                                                     |
| ------------------------------------------------------- | ------------------------------------------------------- |
| <img width="360" src="tests/tiger.svg" alt="Tiger.svg"> | <img width="360" src="tests/tiger.png" alt="Tiger.png"> |

## API

### `new RustySvg(svgStr: string)`

Create a new `RustySvg` instance. This constructor internally runs `usvg` parser to standarize the input file

### `rustySvg.cubic_path_to_quad(tolerance: number)`

Convert all cubic curves in this file to quadratic curves. Useful for font glyph rendering (which only allows quadratic curves)

### `rustySvg.to_string(): string`

Output the svg to a string

### `rustySvg.inner_bbox(): BBox`

Get a tight bbox which removes all transparent space in every directions of the file

### `rustySvg.crop(bbox: BBox)`

Crop the svg with a given bbox. Combined with `.inner_bbox()` API, you could remove the transparent space around the file.

NOTE: this API currently do not actually change the content of the SVG. It uses viewbox to hacky display a certain rect of the file only.

### `getter width(): number`

Width of the file

### `getter height(): number`

Height of the file
