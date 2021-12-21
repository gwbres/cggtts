## CGGTTS
CGGTTS Rust package todo list 

### CRC 
- [ ] use and verify Cggtts::header CRC field 
- [x] use and verify Cggtts::track CRC field 

### File writer
- [ ] provide a file writer interface
 - [x] test system delay interface

### Dual frequency
- [x] support all features for dual frequency receivers
 - [x] file must comply with advanced delay knowledge 
 - [ ] frame descriptor must be parsed & used 
 - [x] watch for INT DLY specs in this situation

### Cggtts::
- [x] simplify IMS/RCVR with a single scan_fmt! mismatch detection
- [ ] use unit labels for generic Cggtts::track scaling

### Cggtts::track
- [x] use from_str() instead of new()
- [x] provide a default constructor
- [x] expose and make easier to external use

### Documentation
- [x] provide some use cases 
- [x] doc on system delays 
