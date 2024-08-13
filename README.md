# dbus-client

[![CI Status](https://github.com/ModProg/dbus-client/actions/workflows/test.yaml/badge.svg)](https://github.com/ModProg/dbus-client/actions/workflows/test.yaml)
[![Crates.io](https://img.shields.io/crates/v/dbus-client)](https://crates.io/crates/dbus-client)
[![Docs.rs](https://img.shields.io/crates/v/template?color=informational&label=docs.rs)](https://docs.rs/dbus-client)
[![Documentation for `main`](https://img.shields.io/badge/docs-main-informational)](https://modprog.github.io/dbus-client/dbus_client/)

## D-Bus Types

For more details on the D-Bus type system see the [D-Bus Specification](https://dbus.freedesktop.org/doc/dbus-specification.html#type-system).

### Simple Types 
| D-Bus Name           | Rust          |
| -------------------- | ------------- |
| b**y**te             | [`u8`]        |
| **b**oolean          | [`bool`]      |
| int16 **n**          | [`i16`]       |
| uint16 **q**         | [`u16`]       |
| **i**nt32            | [`i32`]       |
| **u**int32           | [`u32`]       |
| **d**ouble           | [`f64`]       |
| **h**andle (UNIX_FD) | [`File`]      |
| **s**tring           | [`String`]    |
| **o**bject_path      | [`Path`]      |
| si**g**nature        | [`Signature`] |

### Complex Types

| D-Bus Name                            | Signature             | Rust                |
| ------------------------------------- | --------------------- | ------------------- |
| st**r**uct                            | `(` *t1* *t2* ... `)` | [`(t1, t2, ...)`]   |
| **a**rray                             | `a` *t*               | [`Vec<t>`]          |
| **v**ariant                           | `v`                   | [`Variant`]         |
| **a**rray containing dict_**e**ntries | `a{` *t1* *t2* `}`    | [`HashMap<t1, t2>`] |

### `DbusObject`

Any Rust type implementing [`DbusObject`] can be used as a type in the macro as well. When used in the return type, prefix with `@`.

### Types implementing [`Append`] / [`Get`]
Types implementing [`Append`] / [`Get`] can be used as well. Use [`#[dbus_dict(t)]`] to map *named structs* to `a{s t}` (if `t` parameter is omitted, **v**ariant is used instead). [`#[dbus_struct(t1 t2 ...)]`] will map both *named* and *tuple structs* to `(t1 t2 ...)` using the types of the fields, unless a different type using the optional arguments `t1`, `t2`, ... are specified. 

[`u8`]: https://doc.rust-lang.org/std/primitive.u8.html
[`bool`]: https://doc.rust-lang.org/std/primitive.bool.html
[`i16`]: https://doc.rust-lang.org/std/primitive.i16.html
[`u16`]: https://doc.rust-lang.org/std/primitive.u16.html
[`i32`]: https://doc.rust-lang.org/std/primitive.i32.html
[`u32`]: https://doc.rust-lang.org/std/primitive.u32.html
[`f64`]: https://doc.rust-lang.org/std/primitive.f64.html
[`File`]: https://doc.rust-lang.org/std/fs/struct.File.html
[`Path`]: https://docs.rs/dbus/latest/dbus/strings/struct.Path.html
[`Signature`]: https://docs.rs/dbus/latest/dbus/strings/struct.Signature.html
[`(t1, t2, ...)`]: https://doc.rust-lang.org/std/primitive.tuple.html
[`Vec<t>`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
[`Variant`]: https://docs.rs/dbus/latest/dbus/arg/struct.Variant.html
[`HashMap<t1, t2>`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html
[`DbusObject`]: TODO
[`Append`]: https://docs.rs/dbus/latest/dbus/arg/trait.Append.html
[`Get`]: https://docs.rs/dbus/latest/dbus/arg/trait.Get.html
[`#[dbus_dict(t)]`]: TODO
[`#[dbus_struct(t1 t2 ...)]`]: TODO
