# Chunk File Format
The contents of a world are stored under a single directory which shares the same name as the world. Inside this directory there is exactly 1 file per 32×32×32 chunk which has been generated in this world. Each such file is known as a "chunk file."

## File name
Each chunk file is named according to the following format:

`{dimension_id}_{chunk_x}_{chunk_y}_{chunk_z}.chunkdata`

A breakdown of this format:
- `dimension_id` is any string of legal characters *except for the underscore (`_`)*. This field is a future-proofing measure in the game features multiple dimensions in a future version.
  - **Note:** There is currently only one supported dimension (the "overworld"). This dimension has the identifier `overworld`, **and that is the only legal value of `dimension_id` at this time.** 
- Each of `chunk_x`, `chunk_y` and `chunk_z` is any combination of digits 0-9, potentially preceded by a dash (`-`) to indicate a negative integer. These fields define to the position of the chunk in the world.

Examples of legal chunk file names:
- `overworld_0_0_0.chunkdata`
- `overworld_-12_3_456.chunkdata`

## File content
The contents of the chunk files are in a type of JSON format.

**Note:** Any position which does not have a defined block is assumed to have the default *air block*. There is no other valid way to represent an air block in these files.

TODO: Finish this spec after implementing file saving