# CGGTTS
CGGTTS Rust package todo list 

## `Cggtts`
- [ ] forbid to add calibrated delay if this
kind of delay is already declared, or overwrite previous value
- [ ] use and verify Cggtts::header CRC field 
- [x] use and verify Cggtts::track CRC field 
- [ ] provide a file writer interface
 - [x] test system delay interface
- [x] simplify IMS/RCVR with a single scan_fmt! mismatch detection
- [ ] use unit labels for generic Cggtts::track scaling

## Dual frequency
- [ ] support all features for dual frequency receivers
 - [x] file must comply with advanced delay knowledge 
 - [ ] frame descriptor must be parsed & used 
 - [ ] improve Delay definitions, we must have a SystemDelay for both carrier frequencies
