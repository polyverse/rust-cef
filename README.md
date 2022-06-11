# DEPRECATION NOTICE

Please note that this repository has been deprecated and is no longer actively maintained by Polyverse Corporation.  It may be removed in the future, but for now remains public for the benefit of any users.

Importantly, as the repository has not been maintained, it may contain unpatched security issues and other critical issues.  Use at your own risk.

While it is not maintained, we would graciously consider any pull requests in accordance with our Individual Contributor License Agreement.  https://github.com/polyverse/contributor-license-agreement

For any other issues, please feel free to contact info@polyverse.com

---

![Build Status](https://github.com/polyverse/rust-cef/workflows/Build%20Status/badge.svg)

# rust-cef

A simple trait that allows any Rust item (struct, enum, etc.) to be serialized as a
[ArcSight Common Event Format](https://community.microfocus.com/t5/ArcSight-Connectors/ArcSight-Common-Event-Format-CEF-Implementation-Standard/ta-p/1645557) string.

## Usage

```.rust
let result = example.to_cef();
```
