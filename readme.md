# Moxide

`Moxide` is a cli version of Cheat Engine, following in the path of [`memo`](https://github.com/lonami/memo), while making some different design choices and a piece of cli intergace.

The original repo provides a thorough tutorial on how to build it from scratch. Be sure to check it.

## A few design choices

- `memo` uses `unsafe` functions as long as they prove useful, and use generic for datatypes, while `Moxide` tries its best to avoid `unsafe` calls.
- `Moxide` uses an enum to represent the data type. The choice is made for a reason: Using generic is indeed more efficient and enum could waste tons of extra space due to its excessive size, but it is much easier to be integrated with a user interface. I'm a bit limited in the knowledge of how to provide a flexible interface for controlling generic types with user inputs.

TODO 1: This enum must be refactored. To search for variable length pattern this enum surely will blow itself up. I found I may end up with something like my Compiler Principle project, an enum indicating the type and a void* pointer to its data.
