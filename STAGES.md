# How the parser is built, stage by stage

The [checkpoints/](checkpoints/) folder holds nine snapshots of the parser. Each one adds a single idea on top of the one before. Stage 9 contains everything that the real app ([src/main.rs](src/main.rs)) uses: `parse`, `Value`, and `ReadingConfig`.

Every stage folder has the same layout:

| File | What it is |
| --- | --- |
| `lib.rs` | The parser as it stands at this stage. |
| `run.rs` | The same parser plus a `main` that runs a fixed list of sample inputs. |
| `results.txt` | The output `run.rs` prints, one `input <TAB> result` line per sample. |
| `tests.rs` / `lesson_test.rs` | Unit tests (stages 1 and 3 only). |

To run any stage on its own:

```sh
rustc --edition=2024 checkpoints/03-recursive/run.rs -o checkpoint && ./checkpoint
```

---

## The big picture

```
01 one item ─► 02 split on commas ─► 03 recursive reader      (structure)
                                          │
04 whitespace + true/false/null ◄─────────┘
05 strings + objects                                            (more value kinds)
06 full numbers (fractions, exponents)
07 simple escapes  \n \t \" …                                   (string details)
08 unicode escapes \uXXXX + surrogate pairs
09 application layer: ReadingConfig ─► src/lib.rs + src/main.rs (using the parser)
```

Stages 1–2 are deliberately naive attempts. Stage 3 is the turning point: it introduces the recursive-descent design that every later stage keeps. Stages 4–8 only add new value kinds or finer grammar rules to that design. Stage 9 leaves the parser alone and adds a layer that *uses* it.

### How the sample results change

Stages 4–9 all run the same 13 inputs, so you can watch each stage turn one more line from ✗ to ✓.

✓ = parsed successfully · ✗ = error · *(correct)* = this input is invalid JSON, so the error is the right answer

| Input | 04 | 05 | 06 | 07 | 08 | 09 |
| --- | :-: | :-: | :-: | :-: | :-: | :-: |
| `[]` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `[12,3]` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `[1,[2,3],4]` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| ` [1, [2,3] ] ` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `[true,null,false]` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `{"x":1,"items":[2,3]}` | ✗ | **✓** | ✓ | ✓ | ✓ | ✓ |
| `"hello"` | ✗ | **✓** | ✓ | ✓ | ✓ | ✓ |
| `1.25e-2` | ✗ | ✗ | **✓** | ✓ | ✓ | ✓ |
| `"a\nb"` | ✗ | ✗ | ✗ | **✓**¹ | ✓ | ✓ |
| `"A😀"` | ✗ | ✗ | ✗ | ✗ | **✓** → `"A😀"` | ✓ |
| `[1,]` | ✗ *(correct)* | ✗ | ✗ | ✗ | ✗ | ✗ |
| `[01]` | ✗ *(correct)* | ✗ | ✗ | ✗ | ✗ | ✗ |
| `[1]x` | ✗ *(correct)* | ✗ | ✗ | ✗ | ✗ | ✗ |

¹ Stage 7's code decodes this escape, but its `results.txt` still shows the stage 6 output. See [Inconsistencies found](#inconsistencies-found).

---

## Stage 1 — [01-one-item](checkpoints/01-one-item/lib.rs)

**Goal:** Parse an array that holds zero or one integer.

**How it works:**
1. Check that the input starts with `[` and ends with `]`.
2. Take the text between them.
3. If it's empty, return `[]`. Otherwise parse the whole thing as one `i64`.

**Returns** `Result<Vec<i64>, &'static str>`, a flat list of integers with a plain text error.

**Results:** `[]` and `[12]` work. `[12,3]` fails because `"12,3"` isn't a single integer.

**What it teaches:** This is the smallest thing that can work. It runs into a wall as soon as there are two items.

---

## Stage 2 — [02-split](checkpoints/02-split/run.rs)

**What's new:** Split the inside text on `,` and parse each piece as an integer.

**Results:** `[12,3]` now works. `[1,[2,3],4]` still fails: splitting on commas gives `"1"`, `"[2"`, `"3]"`, `"4"`, and the nested array gets torn apart.

**What it teaches:** This stage exists to **show why naive splitting fails**. A comma inside a nested array belongs to the inner array, not the outer one, and string splitting can't know that. This is the problem the README describes.

---

## Stage 3 — [03-recursive](checkpoints/03-recursive/lib.rs) (the turning point)

**What's new:** The parser is rebuilt as a **recursive-descent reader**. Every later stage keeps this design.

**New types:**
```rust
enum Value { Number(i64), Array(Vec<Value>) }    // a tree, not a flat list
struct ParseError { offset: usize, expected: &'static str }
```

**New functions:**
| Function | Job |
| --- | --- |
| `parse` | Read one value, then insist the input is finished (`[1]x` → "end of input" error at byte 3). |
| `read_value` | Look at the next byte and dispatch: `[` → array, `-`/digit → number. |
| `read_number` | Read an optional `-` and then digits. Reject leading zeros like `01`. Convert to `i64` and reject overflow. |
| `read_array` | Read `[`, then repeatedly call **`read_value`** for each item, then expect `,` or `]`. |

**The key idea:** All the readers share one cursor, `pos: &mut usize`. When `read_array` meets a nested `[`, it calls `read_value`, which calls `read_array` again. The inner call consumes `[2,3]` and leaves `pos` just after its `]`, so the outer array picks up at the next `,`. Each comma is handled by whichever array owns it.

**Errors now carry a byte offset**, e.g. `[1,]` → `offset: 3, expected: "an integer or an array"`.

**Tests:** [tests.rs](checkpoints/03-recursive/tests.rs) covers empty, flat, nested, empty-child, negative, missing item, trailing input, leading zero, unfinished input, missing separator, overflow, and empty input.

**Limits:** No spaces, and only integers and arrays.

---

## Stage 4 — [04-whitespace-literals](checkpoints/04-whitespace-literals/lib.rs)

**What's new:**
- `skip_ws` skips space, tab, `\r`, and `\n`. It runs before each value, before each `,`/`]`, and after the top-level value.
- `read_literal` matches an exact spelling byte by byte: `null`, `true`, `false`.
- `Value` gains `Null` and `Bool(bool)`.

**Results:** ` [1, [2,3] ] ` and `[true,null,false]` now parse. Objects, strings, and decimals still fail.

**Whitespace rule (kept from here on):** A value reader skips the whitespace *before* its value and leaves the byte *after* its value alone. The caller (array, object, or `parse`) handles what comes next.

---

## Stage 5 — [05-string-objects](checkpoints/05-string-objects/lib.rs)

The largest jump after stage 3.

**What's new:**
- **Strings:** `read_string` reads up to the closing `"`. Any backslash is rejected for now (`InvalidEscape`), and so are raw control characters (`UnescapedControl`).
- **Objects:** `read_object` reads `{`, then repeats `"key"` `:` value, separated by `,`, until `}`. Members are stored as `Vec<(String, Value)>`, which **keeps source order and duplicate keys**.
- **Structured errors:** `ParseError.expected: &str` becomes `ParseError.kind: ErrorKind`, an enum (`Expected`, `InvalidEscape`, `InvalidUnicodeEscape`, `UnescapedControl`, `InvalidNumber`, `TrailingInput`, `NestingLimit`). Some variants aren't used until later stages.
- **Nesting limit:** `read_value` takes a `depth` argument. Opening a 129th nested `[` or `{` returns `NestingLimit`, so hostile input like `[[[[…` can't overflow the call stack.
- Readers now take `&str` instead of `&[u8]`, so they can slice substrings out directly.
- `read_digits` is split out as a helper.

**Results:** `{"x":1,"items":[2,3]}` and `"hello"` now parse. `1.25e-2` fails with `TrailingInput` at byte 1, because `1` was read as an integer and `.` was left over.

---

## Stage 6 — [06-full-numbers](checkpoints/06-full-numbers/lib.rs)

**What's new:**
- `read_number` follows the full JSON number grammar: an optional `-`, then either `0` or a nonzero digit followed by more digits, then an optional `.digits` fraction, then an optional `e`/`E` exponent with an optional sign.
- **Numbers are stored as validated text**, not converted:
  ```rust
  pub struct Number(String);       // e.g. Number("1.25e-2")
  impl Number {
      fn as_str(&self) -> &str     // the exact original spelling
      fn as_i64(&self) -> Result<i64, …>  // explicit conversion that can fail
  }
  ```
  This means large numbers are never rounded silently, and the caller decides how to convert.

**Results:** `1.25e-2` → `Number(Number("1.25e-2"))`. Output elsewhere changes from `Number(12)` to `Number(Number("12"))` because of the new wrapper type.

---

## Stage 7 — [07-simple-escapes](checkpoints/07-simple-escapes/lib.rs)

**What's new:**
- `read_string` no longer returns a plain slice. It builds an output `String`. It copies runs of ordinary text in bulk (`run_start..pos`), and when it meets a `\` it appends the decoded character and starts a new run.
- `read_escape` decodes the eight short escapes: `\"` `\\` `\/` `\b` `\f` `\n` `\r` `\t`. Anything else, including `\u`, is still `InvalidEscape`.

**Results:** `"a\nb"` now decodes to a real newline. `\u…` still fails.

---

## Stage 8 — [08-unicode](checkpoints/08-unicode/lib.rs)

**What's new:**
- `read_escape` routes `\u` to `read_unicode`.
- `read_hex4` reads exactly four hex digits into a `u32`.
- `read_unicode` handles **UTF-16 surrogate pairs**:
  - A normal code point like `A` becomes `A` directly.
  - A high surrogate (`D800–DBFF`) **must** be followed by `\u` plus a low surrogate (`DC00–DFFF`). The two combine as `0x10000 + ((hi − 0xD800) << 10) + (lo − 0xDC00)`.
  - A lone low surrogate, or a high surrogate with no partner, is rejected with `InvalidUnicodeEscape`, because a Rust `String` can't hold one.

**Results:** `"A😀"` → `"A😀"`. With this stage, the parser supports all of JSON.

---

## Stage 9 — [09-application](checkpoints/09-application/lib.rs)

**What's new:** The parser itself is unchanged, and `results.txt` matches stage 8. This stage adds module docs that state the parser's policies, plus an **application layer** that sits on top of the parser:

```rust
pub struct ReadingConfig { pub enabled: bool, pub readings: Vec<Value> }
pub enum ConfigError { ExpectedObject, Missing(&str), Duplicate(&str), WrongType(&str) }
```

`ReadingConfig::from_value` checks a parsed `Value` against one specific shape:
- The top-level value must be an object.
- `enabled` must exist and be a bool. `readings` must exist and be an array.
- Either field appearing twice is an error, even though the *parser* allows duplicate keys.
- Unknown keys are ignored.

**What it teaches:** **Valid JSON is not the same as valid configuration.** Syntax checks belong to the parser, and schema rules belong to the application. The two have different policies (for example on duplicate keys), and keeping them separate keeps each one simple.

---

## After stage 9: `src/` and [src/main.rs](src/main.rs)

The finished crate (`json_step_by_step`) turns stage 9 into a real library and program:

| File | Role |
| --- | --- |
| [src/lib.rs](src/lib.rs) | The full parser plus `ReadingConfig`, exported as a library. |
| [src/main.rs](src/main.rs) | The command-line app. |
| [examples/validate.rs](examples/validate.rs) | A syntax-only checker that reports through its exit code. |
| [tests/parser.rs](tests/parser.rs) | Integration tests. |

What `main.rs` does, step by step:

1. **Get the file path** from the first argument. If it's missing, print usage and **exit 2**.
2. **Read the file** as UTF-8. If it can't be read, **exit 2**.
3. **`parse(&input)`**: on a syntax error, print `JSON error at byte N: Kind` and **exit 1**.
4. **`ReadingConfig::from_value(value)`**: if the JSON is valid but has the wrong shape, print `Configuration error: …` and **exit 1**.
5. **Print the result**: the `enabled` flag, how many outer readings there are, and, if the second reading is an array, its length and each number's original spelling (`number.as_str()`).

With [example.json](example.json) (`{"readings":[1,[2,3],4],"enabled":true}`):

```text
Enabled: true
Outer readings: 3
Second reading contains 2 items
Number: 2
Number: 3
```

"Outer readings: 3" is exactly the case stage 2 got wrong and stage 3 fixed.

---

## Inconsistencies found

I noticed these while reading the stages. I left them unchanged:

1. **The folder is named `checkpoints/`, not `stages/`.** Git still shows an old `stages/` folder, but it isn't on disk anymore.
2. **[02-split/lib.rs](checkpoints/02-split/lib.rs) won't compile:** `inside..split(',')` has a doubled dot (should be `inside.split(',')`). Its header comment also still says "checkpoint 1". The [run.rs](checkpoints/02-split/run.rs) version is correct.
3. **[07-simple-escapes/results.txt](checkpoints/07-simple-escapes/results.txt) looks out of date.** It is identical to stage 6's file. Based on the code, `"a\nb"` should print `Ok(String("a\nb"))`, and the `\u` line should report `InvalidEscape` at offset **2** rather than 1. `check_checkpoints.py` will most likely flag this stage. I worked this out from the code and didn't run it.
4. **`check_checkpoints.py` looks for `lesson_tests.rs`**, but stage 1's file is `lesson_test.rs` (no `s`), so those tests never run.
5. **In stage 1's `flat_is_not_yet_supported` test**, the input `"[12, 3"` is missing its `]`. It fails with "expected an array" rather than the case the test name describes.
6. **The README links to `checkpoints/05-strings-objects/`**, but the folder is `05-string-objects`, so that link is broken.
7. **Stage 4's header comment still says "Checkpoint 3"**, and its fallback error still reads "an integer or an array" even though it now accepts literals too.
