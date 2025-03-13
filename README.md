eXperimental Zlib compressed Interleaved Bitmap format
======================================================

These are just my experiments with zlib compression and interleaved bitmaps. I
just want to check how well that compresses. In any case this doesn't support
progressive reading any more than reading any kind of zlib compressed data does.
In that way it is different to PNG.

File Format
-----------

Byte order of all integer values is **little endian**. The encoding of any text
is **UTF-8**.

Types:

| Type   | Size | Description |
| :----- | ---: | :---------- |
| `U8`   |    1 | Unsigned 8-bit integer (a byte). |
| `U16`  |    2 | Unsigned 16-bit integer. |
| `U32`  |    4 | Unsigned 32-bit integer. |
| `U64`  |    8 | Unsigned 64-bit integer. |
| `U128` |   16 | Unsigned 128-bit integer. |
| `F16`  |    2 | 16-bit floating point number. |
| `F32`  |    4 | 32-bit floating point number. |
| `F64`  |    8 | 64-bit floating point number. |
| `F128` |   16 | 128-bit floating point number. |
| `ZSTR` |   >1 | A NUL (zero) terminated UTF-8 string. |

Basic layout:

```
┌─────────────┐
│ ┌─────────┐ │
│ │ Header  │ │
│ └─────────┘ │
│ ┌─────────┐ │
│ │  Chunk  │ │
│ └─────────┘ │
|     ...     │
│ ┌─────────┐ │
│ │  Chunk  │ │
│ └─────────┘ │
└─────────────┘
```

There has to be exactly one `body` chunk.

### Header

| Offset | Type    | Name         | Description |
| -----: | :------ | :----------- | :---------- |
|      0 | `U8[4]` | `file_magic` | `"XZIB"`    |
|      4 | `U8`    | `flags`      | bit 1 ... interleaved<br>bit 2 ... floating-point |
|      5 | `U8`    | `channels`   | Supported values: 1, 3, 4 |
|      6 | `U8`    | `planes`     | Number of planes in interleaved format or number of bits per unit (channel or index value) otherwise.<br>Supported values:<br>Integer:<ul><li>interleaved: 1 ... 8, 16, 32, 64, 128</li><li>non-interleaved: 1, 4, 8, 16, 32, 64, 128</li></ul>Floating-point: 16, 32, 64, 128<br>Indexed: 1 ... 255 |
|      7 | `U8`    | `index_planes` | `0` means not indexed. Otherwise: 8, 16, 32, 64, 128 |
|      8 | `U32`   | `width`      | The width of the image in pixels. |
|     12 | `U32`   | `height`     | The height of the image in pixels. |

### Chunks

| Offset  | Type               | Name           | Description |
| ------: | :----------------- | :------------- | :---------- |
|       0 | `U8[4]`            | `chunk_magic`  | If the first ASCII letter is lower case the `chunk_length` field is `U32`, if it is upper case the field is `U64`. If the sencond ASCII letter is lower case then the payload is zlib compressed, if it is upper case the payload is uncompressed. |
|       4 | `U32` or `U64`     | `chunk_length` | The number of bytes in the payload of this chunk. |
| 8 or 12 | `U8[chunk_length]` | `payload`      | Payload of the chunk. |

#### `meta` Chunk

A list of meta data entries in the form of:

| Offset  | Type   | Name    | Description   |
| ------: | :----- | :------ | :------------ |
|       0 | `U8`   | `key`   | See below.    |
|       1 | `ZSTR` | `value` | Text content. |

There are as many such entries in the chunk as fit. There may be zero padding at the end of the chunk.

| Key | Multiple           | Name         | Description |
| --: | :----------------: | :----------- | :---------- |
| `1` |                    | `title`      | Title of the image. |
| `2` |                    | `created_at` | `YYYY-MM-DD` or `YYYY-MM` or `YYYY` |
| `3` | :heavy_check_mark: | `author`     | Author name. |
| `4` | :heavy_check_mark: | `license`    | [SPDX ID](https://spdx.org/licenses/) |
| `5` | :heavy_check_mark: | `links`      | HTTP URL(s) to where to find this work or more about this work on the internet. |
| `6` |                    | `comment`    | A longer comment or description, probably multi-line. |

#### `xmet` Chunk

Similar to `meta`, but extensible. Per default the keys are from the
[Dublin Core](https://en.wikipedia.org/wiki/Dublin_Core) meta-data vocabular,
but you can specify other vocabularies by having an entry with a key like
`xmlns:x` and the value `http://example.com/some/other/metadata/spec` and then
use keys like `x:foo` to referre to that meta-data specification.

| Offset | Type   | Name    | Description   |
| -----: | :----- | :------ | :------------ |
|      0 | `ZSTR` | `key`   | Dublin core or other metadata standard field name. |
|      ? | `ZSTR` | `value` | Text content. |

There may be zero padding at the end of the chunk.

#### `indx` Chunk

This chunk is required if `index_planes` is non-zero. It also may only
exist if `index_planes` is non-zero.

```
number_of_colors = chunk_length / (channels * index_planes)
```

The colors are just layed out in `L`, `RGB`, or `RGBA` format, one color
after the other. Just like the non-interleaved `body` format.

#### `body` Chunk

##### Non-Interleaved

The color values (or indices) are just layed out in standard `L`, `RGB`, or
`RGBA` format, the color of one pixel after another, row by row.

The bits of a row of a non-indexed 3 channel 4 bits/channel image with a width
of 5 would be layed out like this:

```
pixel
    1  R1 R2 R3 R4 G1 G2 G3 G4 B1 B2 B3 B4
    2  R1 R2 R3 R4 G1 G2 G3 G4 B1 B2 B3 B4
    3  R1 R2 R3 R4 G1 G2 G3 G4 B1 B2 B3 B4
    4  R1 R2 R3 R4 G1 G2 G3 G4 B1 B2 B3 B4
    5  R1 R2 R3 R4 G1 G2 G3 G4 B1 B2 B3 B4
```

##### Interleaved

The bits of a row of a non-indexed 3 channel 4 bits/channel image with a width
of 5 would be layed out like this:

```
pixel:  1  2  3  4  5 padding
       R1 R1 R1 R1 R1 0 0 0
       G1 G1 G1 G1 G1 0 0 0
       B1 B1 B1 B1 B1 0 0 0
       
       R2 R2 R2 R2 R2 0 0 0
       G2 G2 G2 G2 G2 0 0 0
       B2 B2 B2 B2 B2 0 0 0
       
       R3 R3 R3 R3 R3 0 0 0
       G3 G3 G3 G3 G3 0 0 0
       B3 B3 B3 B3 B3 0 0 0
       
       R4 R4 R4 R4 R4 0 0 0
       G4 G4 G4 G4 G4 0 0 0
       B4 B4 B4 B4 B4 0 0 0
```

This means each plane needs to be padded to a multiple of 8 bits. So there is some
waste there. But this experiment is to see if grouping the most/least significant
bits like that improves zlib compression. The assumtion is that the most
significant bits will be the same in large areas of an image and thus a row of e.g.
just 0s will compress well. But it also increases the entropy for the least
siginificant bits. The padding bits don't need to be 0. It's probably better to
use whatever value the bits before have in order to reduce entropy.

#### `foot` Chunk

Has to be the last chunk.

| Offset | Type    | Name            | Description   |
| -----: | :------ | :-------------- | :------------ |
|      0 | `U8`    | `checksum_type` | See below.    |
|      1 | `U8[?]` | `checksum`      | Checksum of the whole file up to, but not including the `foot` chunk. |

| Checksum Type | Payload Length | Name    |
| ------------: | -------------: | :------ |
|             0 |              4 | CRC32   |
|             1 |             20 | SHA-1   |
|             2 |             28 | SHA-224 |
|             3 |             32 | SHA-256 |
|             4 |             48 | SHA-384 |
|             5 |             64 | SHA-512 |

