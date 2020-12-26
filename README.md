![Build Status](https://github.com/polyverse/rust-cef/workflows/Build%20Status/badge.svg)

# rust-cef

A simple trait that allows any Rust item (struct, enum, etc.) to be serialized as a
[ArcSight Common Event Format](https://community.microfocus.com/t5/ArcSight-Connectors/ArcSight-Common-Event-Format-CEF-Implementation-Standard/ta-p/1645557) string.

## Usage

```.rust
let result = example.to_cef();
```
