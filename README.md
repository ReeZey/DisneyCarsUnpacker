# Disney Cars (2006) Video Game Decompiler

# what this
- Decompiles `.pak` files. which are basically just weird zip files
- Converts `.wav (IMA 4bit ADPCM)` to normal `.wav` files  
- **TODO:** Convert images (Unpack working, not repacking and some formats still unknown)

# the custom formats
this game loves its weird formats, the following are what i've found from my own research of the game files, which might be useful for someone, probably  not

## `.pak` packed files: 
|format| **.PAK** |
| ----------- | ----------- |
| u32 | number of boxed files |
| boxes | boxes are formatted: <br> **[Filename + Offset + Size]** |
| data | the actual data |

- Each Meta-block is 108 bytes, filename is 100 bytes, offset is 4 bytes, size is 4 bytes.
- Filename can only be up to 100 chars, is padded by 0xCC and null terminated.  

## `.wav (ADPCM)` audio files:
|format| .wav (ADPCM) |
| ----------- | ----------- |
|header| First 16 bytes normal wav stuff, such as `RIFF` header, total size & `WAVEfmt `|
|u32| Chunk size|
|u16| format = 2 which is "ADPCM" but audio players get confused cause they think its `MS ADPCM` |
|u16| channel count (1-2)|
|u32| sample rate|
|u32| byte rate |
|u16| block align|
|u16| bits per sample |
|u32| some weird constant = 4194304, which is always same in all audio files (checksum?) |
|footer| `data` footer + size of data |
|data| the actual data |

- very simular to an normal `.wav`file with some weird differences.
- all the conversion does in reality is changing format to 1 (PCM), and convert the data from ADPCM to normal PCM, leaving most of the file the same


## `.dxt` image files:
|format| .dxt |
| ----------- | ----------- |
|u32| the number 2, written in the beginning of each file |
|u32| image format (54, 38, 50.. etc), these are which compression is used DXT1/DXT2/DXT3 |
|u32| unknown (more research needed) |
|u32| number of images (if there are mipmaps of an image), if = 1 no mipmaps, only one image|
|u32| width of highest resolution |
|u32| height of highest resolution |

following part is repeated for each **number of images**
|format| .dxt |
| ----------- | ----------- |
|u32| current image width|
|u32| current image height|
|u32| size of current image|
|data| the actual data|  
- VERY EXPERIMENTAL RIGHT NOW
- DOES NOT PACK CORRECTLY WHEN USED, **USE ONLY WHEN EXTRACTING ASSETS FOR NOW**  
  

# why  
bored

# how to use?
either build with `cargo b -r` or run it directly with `cargo r -r -- --mode X/XR/R`  
use `-h` for usage