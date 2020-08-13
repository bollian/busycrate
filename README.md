# BusyCrate

BusyBox but written in Rust for fun.

FYI, https://github.com/samuela/rustybox is a thing and you should prefer it over this.
I just work on this when I'm tired of working on all my other projects. I see it as
a way to learn the Unix utilities.

# Why even bother

Rust binaries tend to be significantly larger than their C or C++ equivalents
since they include a lot of extra ~~bloat~~ features, with backtracing and
`std::fmt` being two large examples. If every single Unix utility were rewritten
in Rust, this cost would be repeated several times over.

There are ways of reducing the extra binary size, but doing something like removing
the standard library can be painful to work with. Instead, BusyCrate combines
several utilities into a single binary to reduce the final cost.
