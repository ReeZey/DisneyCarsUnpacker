# Disney Cars 2005 Video Game Decompiler

# what this
- Decompiles `.pak` files.

# what are `.pak` files?  
idk i just made some guesses in how files were packed.

## the format
i guess i can put my findings here about the format


|  | **.PAK** |
| ----------- | ----------- |
| Number | Number of Metadata files |
| Metadata | meta of files such as: <br> **[Filename + Offset + Size]** |
| Data | The actual data |

* Each Meta-block is 108 bytes, filename is 100 bytes, offset is 4 bytes, size is 4 bytes.
* Filename can only be up to 100 chars, is padded by 0xCC and null terminated.
* Audio files are  some weird *Adaptive differential pulse-code modulation*  (ADPCM) file type

# why  
bored

# how to use?
dont