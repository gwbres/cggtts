CGGTTS
======

CGGTTS is the main structure, is can be parsed from a standard CGGTTS file, and can be dumped into a file following standards. 

Cggtts parser 
=============

## Known behavior 

* The parser supports current revision **2E** only, it will reject files that have different revision number.
* This parser does not care for file naming conventions

* While standard specifications says header lines order do matter,
this parser is tolerant and only expects the first CGGTT REVISION header to come first.

* BLANKs between header & measurements data must be respected
* This parser does not care for whitespaces, padding, it is not disturbed by their abscence
* This parser is case sensitive at the moment, all data fields and labels should be provided in upper case,
as specified in standards

* We accept several \"COMMENTS =\" lines, although it is not specified in CGGTTS. Several comments are then parsed. 

Notes on System delays and this parser

* This parser follows standard specifications, 
if \"TOT\" = Total delay is specified, we actually discard
any other possibly provided delay value, and this one superceeds
and becomes the only known system delay.
Refer to the System Delay documentation to understand what they
mean and how to specify/use them.
