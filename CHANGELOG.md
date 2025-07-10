# Changelog

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
