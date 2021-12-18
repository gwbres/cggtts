## CGGTTS
CGGTTS Rust package todo list 

### CRC 
- [ ] use and verify Cggtts::header CRC field 
- [ ] use and verify Cggtts::track CRC field 

### File writer
- [ ] provide a file writer interface
 - [ ] test system delay interface

### Dual frequency
- [ ] support all features for dual frequency receivers
 - [ ] file must comply with advanced delay knowledge 
 - [ ] frame descriptor must be parsed & used 

### Cggtts::track
- [x] use from_str() instead of new()
- [x] provide a default constructor
- [x] expose and make easier to external use

### Documentation
- [x] provide some use cases 
- [ ] doc on system delays 
