Track: CGGTTS measurements
==========================

CGGTTS measurements (referred to as _tracks_) are Common View realizations.

Two classes of measurement exist:
* `CommonViewClass::Single`
* `CommonViewClass::Combination`

A track comprises several data fields, refer to the crate official documentation 
for their definition.

```rust
let first = cggtts.tracks.first()
    .unwrap();
assert_eq!(first.elevation, 1E-9);
assert_eq!(first.azimuth, 1E-10);
```

Follows BIPM tracking duration specifications: 

```rust
assert_eq!(first.follows_bipm_specs(), true);
```
