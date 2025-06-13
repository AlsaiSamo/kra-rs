
# Table of Contents

1.  [Introduction](#orgd4300e9)
    1.  [Features](#org2e5307c)
        1.  [Parse meta](#org8e9aa58)
    2.  [Tasks](#orgce7c4e1)
        1.  [Decode raster data](#org2a3f14c)
        2.  [Process raster layers](#orge22b7ad)
        3.  [Create a `.kra` file](#org106e343)
2.  [License](#orgde8dee3)


<a id="orgd4300e9"></a>

# Introduction

Parsing `.kra` files.
For now, `kra-file` only parses the metadata of a file.
The goal is to allow loading the files or creating them from scratch, and modifying them (updating node data, inserting and removing nodes, updating metadata)
The project will be implemented in two crates, one providing core library functionality, another - file access.


<a id="org2e5307c"></a>

## Features


<a id="org8e9aa58"></a>

### Parse meta

-   File metadata
-   Layer metadata
    Does not parse `selected="true"` property, which indicates the layer that should be selected when loading the file.


<a id="orgce7c4e1"></a>

## Tasks


<a id="org2a3f14c"></a>

### CURRENT Decode raster data

This will be done in `kra` crate.


<a id="orge22b7ad"></a>

### NEXT Process raster layers

In `kra-file`.

-   Decode
-   Encode


<a id="org106e343"></a>

### NEXT Create a `.kra` file

-   Roundtrip a file
-   Create a file from scratch


<a id="orgde8dee3"></a>

# License

The library is licensed under GPL 3.0.

