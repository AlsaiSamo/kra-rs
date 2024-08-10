
# Table of Contents

1.  [Introduction](#orgb67aec6)
    1.  [Features](#org2140df0)
        1.  [Parser](#orgf35dc31)
    2.  [Tasks](#org59f458e)
        1.  [Restructure](#orge0de465)
        2.  [Functionality](#org3dc8757)
        3.  [Pre-publish](#org5bd6c0b)
2.  [License](#org0abccc8)


<a id="orgb67aec6"></a>

# Introduction

Reimplementation of Krita types and functions, and support for `.kra` files.


<a id="org2140df0"></a>

## Features


<a id="orgf35dc31"></a>

### Parser

-   [-] Parsing metadata
    -   [X] Builds the image as a tree of nodes
    -   [ ] It does not register what layer was selected at the moment of saving (attribute `selected="true"`).
-   [ ] Extracting images


<a id="org59f458e"></a>

## Tasks


<a id="orge0de465"></a>

### CURRENT Restructure

Currently, only loader is implemented, and it only works with the XML metadata.
Loader should be split away into `kra_file`, while `kra` reimplements core Krita items.


<a id="org3dc8757"></a>

### NEXT Functionality

-   [ ] Core types
    Composition operators/filters, color, etc.
-   [ ] Configured loader
    Toggle data loading, image converting, SVG interpreting, etc.


<a id="org5bd6c0b"></a>

### NEXT Pre-publish

-   [ ] Tests
-   [ ] Examples
-   [ ] Docs


<a id="org0abccc8"></a>

# License

The library is licensed under GPL 3.0 as it is a rewrite of Krita&rsquo;s source code.

