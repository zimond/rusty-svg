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
