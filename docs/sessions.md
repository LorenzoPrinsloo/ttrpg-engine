# Sessions

A session is one game's saved state: what's left in every bag, and which rows of each unique table have come up. Sessions mean a bag that has given out three encounters still has seven left next week, and a unique artifact found in one game stays found.

## Picking a session

```sh
ttrpg-engine --session derelict          # interactive, in the "derelict" session
ttrpg-engine --session=derelict draw art # one-shot, same session
ttrpg-engine draw art                    # no flag: the session called "default"
```

`--session` has to come before the command, so text inside a command is never mistaken for it. Names can use letters, digits, `-` and `_`. Anything else, such as `../elsewhere`, is refused:

```text
"../x" is not a session name, use letters, digits, `-` and `_`
```

## When it saves

After every command that changes a bag or table (`draw`, `refill` and `reset`), the session is saved straight away. Quitting any way at all, even closing the terminal, loses nothing. Commands that only look (`tables`, `roll`, `render`, `help`) don't save.

```text
$ ttrpg-engine --session derelict draw encounters 3
encounters: 3 Escaped Child Android(s)
encounters: 5 Security Android(s) (pg. 10)
encounters: 4 Former CLOUDBANK Research Scientist(s)
$ ttrpg-engine --session derelict tables
Bags
  artifacts    100 left  unique
  encounters     7 left
Tables
  rumours        5 rows
```

A save writes to a temporary file in the same directory, flushes it to disk, then renames it over the old one. The rename is atomic, so a crash part-way through leaves the previous save intact, never half a file. If a save fails, for example because the disk is full, the program reports it and carries on, and the next change tries again.

## Putting things back

| Command | Bags | Tables |
|---|---|---|
| `refill [name]` | Back to full: every marble, in the file's ratios | Every row back in play, if the table is unique. Otherwise there's nothing to put back |
| `reset [name]` | Exactly as the file has it, including any `remaining` counts the file sets | Exactly as the file has it, including rows the file marks `used` |

Without a name, both commands apply to everything.

```text
$ ttrpg-engine --session derelict refill encounters
encounters refilled, 10 available
$ ttrpg-engine --session derelict refill rumours
rumours rolls fresh every time, nothing to put back
$ ttrpg-engine --session derelict reset
everything is back to how its file has it
```

A file can start a bag part-drawn, or a table with rows already used. That's useful for a scenario that begins mid-expedition:

```toml
[[items]]
ratio = 0.3
text = '''[1d10] Escaped Child Android(s)'''
remaining = 1   # only one of the three left

[[rows]]
range = "10"
text = '''{bright cyan|**Something answers.**}'''
used = true     # already happened
```

`remaining` counts marbles after the ratios are reduced, so `0.3` in a bag of ten has at most 3. An entry without `remaining` starts full.

## Where sessions are kept

Each session is one file, `<name>.json`, in:

| | |
|---|---|
| `MOTHERSHIP_SESSIONS` | if set, this directory |
| macOS | `~/Library/Application Support/ttrpg-engine/sessions` |
| Linux | `~/.local/share/ttrpg-engine/sessions` (or `$XDG_DATA_HOME`) |
| Windows | `%APPDATA%\ttrpg-engine\sessions` |

Sessions are kept apart from the config, so drawing doesn't keep changing files you might have in version control. To back up a campaign, or branch it before a risky choice, copy the file. To delete a session, delete the file.

## What's in a session file

A session file has the same shape as the TOML files, as JSON, plus a fingerprint of each config file:

```json
{
  "version": 1,
  "config_hashes": {
    "bags/artifacts.toml": "c6c5ae1bcad5b1bb",
    "bags/encounters.toml": "c06146d1aa9ca249",
    "tables/rumours.toml": "26777e5e94da3bc5"
  },
  "bags": {
    "encounters": {
      "unique": false,
      "items": [
        { "ratio": 0.1, "text": "A Diver", "remaining": 1 },
        { "ratio": 0.1, "text": "Ghost in the Machine", "remaining": 0 }
      ]
    }
  },
  "tables": {
    "rumours": {
      "unique": false,
      "rows": [
        { "range": "1-3", "text": "The **reactor** shudders. Everyone Close takes {red|[1d5] Stress}.", "used": false }
      ]
    }
  }
}
```

(Shortened: a real file lists every item and row.)

## Editing config files between sessions

When a session loads, every bag and table is checked against the file it came from, using a fingerprint of the file's exact bytes (64-bit FNV-1a). The fingerprint is a stable hash, so a Rust upgrade can't make every save look out of date.

| If the file… | the bag or table… |
|---|---|
| hasn't changed | carries on from the session |
| was edited (even a comment) | starts as the file now has it, with a note |
| was deleted | is dropped from the session, with a note |
| is new | starts as the file has it |

A saved bag can't safely be laid over an edited file, because the file may have new entries or different ratios. So an edited bag starts full and tells you:

```text
bags/encounters.toml changed since the last save, encounters starts full
tables/rumours.toml is gone, the saved rumours is dropped
```

Notes go to stderr, so a one-shot command's stdout is only its answer.

## When a session can't be read

If a session file exists but can't be read, the program stops instead of starting fresh. Carrying on would overwrite the saved game on the first draw.

```text
/path/to/sessions/broken.json: EOF while parsing an object at line 2 column 0
/path/to/sessions/future.json: saved by a newer version (format 2, this one reads up to 1)
```

The version check matters: a file from a newer build might hold fields this build doesn't know about, and saving over it would quietly erase them. Fix or move the file, or use a different session name.
