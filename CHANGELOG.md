# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/) and this project
adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

### CHANGED

-   `Callback`s can now have empty values (constructed by `Default::default()`), obviating the need
    to wrap them in `Option<>`. The coercion for `Option<Callback<>>` for functions has been
    replaced with one for just `Callback<>` as a consequence, which means you'll have to update your
    subcomponents: just replace any `Option<Callback<A>>` with `Callback<A>` in your properties, and
    remove the `Some` check on `self.on_my_callback.send()`—you can just call `send()` on an empty
    callback directly now, and it will quietly do nothing. If you'd rather not needlessly construct
    the value for `send()` when a callback is empty, you can use `Callback::is_empty()` as a
    predicate instead of the `Some` check.

### ADDED

-   Properties which want an `Option<&A>` will now accept an `Option<A>`. (#33)

## [0.1.0] - 2020-02-07

Initial release.
