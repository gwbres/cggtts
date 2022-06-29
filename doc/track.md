Track: CGGTTS measurements
==========================

CGGTTS measurements (referred to as _tracks_) are Common View realizations.

Two classes of measurement exist:
* `CommonViewClass::Single`
* `CommonViewClass::Combination`

A track comprises several data fields, refer to the crate official documentation 
for their definition.

## Track manipulation

Access one measurement through `Cggtts.tracks` array:

```rust
let cggtts = Cggtts::from("");
assert_eq!(cggtts.tracks.len(), 10);
let first = cggtts.tracks.first();
assert_eq!(first.class,
```

```rust
assert_eq!(first.follows_bipm_specs(), true);
```
