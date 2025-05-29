
# Table of Contents

1.  [Introduction](#orgd41b15d)
    1.  [Features](#org39ae81a)
        1.  [Parse meta](#org28454d1)
    2.  [Tasks](#org37dc4d7)
        1.  [Refresh metadata parsing](#org0936134)
        2.  [Refresh library interface, docs](#org44ddcf9)
        3.  [Raster layers](#org93fd807)
2.  [License](#orge1247a3)


<a id="orgd41b15d"></a>

# Introduction

Parsing `.kra` files.
For now, `kra-file` only parses the metadata of a file.
The goal is to allow loading the files or creating them from scratch, and editing them.
The project will be later implemented in two crates, one providing core library functionality, another - file access.


<a id="org39ae81a"></a>

## Features


<a id="org28454d1"></a>

### Parse meta

-   File metadata
-   Layer metadata
    Does not parse `selected="true"` property, which indicates the layer that should be selected when loading the file.


<a id="org37dc4d7"></a>

## Tasks


<a id="org0936134"></a>

### CURRENT Refresh metadata parsing

It should actually work.


<a id="org44ddcf9"></a>

### CURRENT Refresh library interface, docs


<a id="org93fd807"></a>

### NEXT Raster layers

-   Extract
-   Decode
-   Encode


<a id="orge1247a3"></a>

# License

The library is licensed under GPL 3.0.

