# brainrust

A [brainfuck](https://en.wikipedia.org/wiki/Brainfuck) interpreter written in Rust, because why not.

## Usage

```shell
cargo run path-to-script.bf
```

## How?

This brainfuck interpreter pre-processes the input program before executing it such that:

* Contiguous increment/decrement pointer commands (e.g. `>` and `<`) are compressed into a single "move pointer" command
that contains the total movement the pointer should go through. e.g. `>>><` is transformed into "move pointer by 2".
* Similarly, contiguous increment/decrement data commands (e.g. `+` and `-`) are compressed into a single "modify data" command.
* Loop related commands are enriched so that the loop start one (`[`) knows the offset in the program to which it should jump to
if the data pointer is 0 (the end of the loop), and the loop end one (`]`) knows where the loop begins so it can jump directly into it. This allows having a flat/non recursive program definition, which is arguably simpler to execute.

### Example

For example, the following nonsensical program:

```
>>++[-]<<
```

Will be transformed into:

```
- Command 0: Move pointer by 2
- Command 1: Modify data by 2
- Command 2: Start loop and jump to command 4 when done
- Command 3: Modify data by -1
- Command 4: End loop, jump to command 2 if loop is not yet done
- Command 5: Move pointer by -2
```
