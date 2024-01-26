
# Table of Contents

1.  [Introduction](#org16cc9fa)
    1.  [Features](#org221adf8)
        1.  [Reading](#orgbed5d66)
        2.  [Writing](#org196ddcc)
        3.  [Rendering](#org705f5a7)
2.  [License](#org213bcdd)


<a id="org16cc9fa"></a>

# Introduction

This is a library for reading `.kra` files.


<a id="org221adf8"></a>

## TODO Features


<a id="orgbed5d66"></a>

### TODO Reading

-   [-] Parsing metadata
    -   [X] Supports these layer types:
        1.  PaintLayer
        2.  GroupLayer
        3.  FilterMask
        4.  SelectionMask
    -   [ ] Does not support these layer types (but adding support should be easy):
        1.  FileLayer
        2.  FilterLayer
        3.  FillLayer
        4.  CloneLayer
        5.  VectorLayer
        6.  TransparencyMask
        7.  TransformMask
        8.  ColorizeMask
    -   [ ] It does not register what layer was selected at the moment of saving (attribute `selected="true"`).
-   [ ] Extracting images


<a id="org196ddcc"></a>

### Writing

Currently, there are no plans to support editing parsed data and writing `.kra` files outside of Krita.
This may change in the future.


<a id="org705f5a7"></a>

### Rendering

Rendering is best be left to a different crate.


<a id="org213bcdd"></a>

# License

The library is licensed under GPL 3.0 because some of its code is or will be adapted from Krita&rsquo;s.

