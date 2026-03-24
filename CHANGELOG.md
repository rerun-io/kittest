# Changelog

## 0.4.0 - 2026-03-24 - accesskit 0.24

* Update accesskit to 0.24.0 and accesskit_consumer to 0.34.0 [#18](https://github.com/rerun-io/kittest/pull/18) by [@delan](https://github.com/delan)
* Update accesskit_consumer to 0.35.0 [#22](https://github.com/rerun-io/kittest/pull/22) by [@SuchAFuriousDeath](https://github.com/SuchAFuriousDeath)

## 0.3.0 - 2025-10-09 - Cloneable `By` and accesskit 0.21

* Make `By` cloneable [#16](https://github.com/rerun-io/kittest/pull/16) by [@lucasmerlin](https://github.com/lucasmerlin)
* Update accesskit to 0.21, accesskit_consumer to 0.30 [#15](https://github.com/rerun-io/kittest/pull/15) by [@DataTriny](https://github.com/DataTriny)
* Update to Rust 2024 edition, bump MSRV to 1.85 [#14](https://github.com/rerun-io/kittest/pull/14) by [@DataTriny](https://github.com/DataTriny)

## 0.2.0 - 2025-06-10 - trait-based Node

`kittest::Node` was removed in favor of `kittest::NodeT`, a trait which you implement for your test frameworks 
node type to unlock the kittest functionality.

This has several advantages:
 - You can add conversions of e.g. `Rect` types to the one native to your framework.
 - You can provide access to data kittest has no knowledge of
 - You can implement event / input helpers that use the types from your framework and match your framework's idioms.

#### Full list of changes:
* Refactor Node to be a trait [#13](https://github.com/rerun-io/kittest/pull/13) by [@lucasmerlin](https://github.com/lucasmerlin)
* Update rust msrv to 1.81 [#10](https://github.com/rerun-io/kittest/pull/10) by [@lucasmerlin](https://github.com/lucasmerlin)
* Bump accesskit to 0.18 [#9](https://github.com/rerun-io/kittest/pull/9) by [@valadaptive](https://github.com/valadaptive)
* Bump accesskit to 0.19 [#11](https://github.com/rerun-io/kittest/pull/11) by [@valadaptive](https://github.com/valadaptive)
* Update msrv to 1.84 [#12](https://github.com/rerun-io/kittest/pull/12) by [@lucasmerlin](https://github.com/lucasmerlin)

## 0.1.0 - 2024-12-16 - Initial release
