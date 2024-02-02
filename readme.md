
# Table of Contents

1.  [Introduction](#org97de986)
    1.  [Features](#orga85e419)
        1.  [Reading](#org6b01810)
        2.  [Writing](#org094713a)
        3.  [Rendering](#orge94b659)
    2.  [Tasks](#orgb0d7304)
        1.  [Publishing](#org345c2dc)
        2.  [Refactoring](#org108473c)
2.  [License](#orgade2f60)


<a id="org97de986"></a>

# Introduction

This is a library for reading `.kra` files.


<a id="orga85e419"></a>

## TODO Features


<a id="org6b01810"></a>

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


<a id="org094713a"></a>

### Writing

Currently, there are no plans to support editing parsed data and writing `.kra` files outside of Krita.
This may change in the future.


<a id="orge94b659"></a>

### Rendering

Rendering is best be left to a different crate.


<a id="orgb0d7304"></a>

## TODO Tasks


<a id="org345c2dc"></a>

### CURRENT Publishing

-   [ ] Docs
-   [ ] Tests
-   [ ] Examples


<a id="org108473c"></a>

### Refactoring

-   [ ] Generate parsing code instead of writing it by hand.


<a id="orgade2f60"></a>

# License

The library is licensed under GPL 3.0 because some of its code is or will be adapted from Krita&rsquo;s.

