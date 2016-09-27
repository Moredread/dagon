# dagon

[![Build Status](https://travis-ci.org/Moredread/dagon.svg?branch=master)](https://travis-ci.org/Moredread/dagon)
[![Coverage Status](https://coveralls.io/repos/github/Moredread/dagon/badge.svg?branch=master)](https://coveralls.io/github/Moredread/dagon?branch=master)
[![Docs Dev](https://img.shields.io/badge/docs-dev-blue.svg)](https://moredread.github.io/dagon/)

An astrophysical hydrodynamics and n-body experiment in Rust.

Very experimental and definitely not stable! 

## Testing for unstable numerical expressions

The [herbie linter](https://github.com/mcarton/rust-herbie-lint) can be
used to look for unstable numerical expressions. To enable linting
during compilation, add the "herbie-lint" feature to your cargo command.