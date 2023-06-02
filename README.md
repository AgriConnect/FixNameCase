FixNameCase
===========

[![crate](https://badgen.net/crates/v/fix-name-case?icon=terminal)](https://crates.io/crates/fix-name-case)


Tool to convert variable and function names in C/C++ source code to [`snake_case`](https://en.wikipedia.org/wiki/Snake_case).

Hidden files and files listed in _.gitignore_ are untouched.

Tested on Ubuntu 22.04+ and Windows 10.

Install
-------

```console
$ cargo install fix-name-case
```

Usage
-----

```console
$ fix-name-case input_folder
```

Requirements
------------

Please install these softwares:

  - [Universal Ctags](http://ctags.io/).
  - [Amber](https://github.com/dalance/amber).

On Windows, please configure so that `ctags` and `ambr` commands can be called without a full path.

Credits
-------

- [Nguyễn Hồng Quân](https://quan.hoabinh.vn)
