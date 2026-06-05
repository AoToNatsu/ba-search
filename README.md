# ba-search

Need to search for hard-to-find phrases like "Nameless Gods"? Or do you want to count how many times "romantic" is said by Natsu?

This project lets you locally scrape the following sections sourced from the [Blue Archive Wiki](https://bluearchive.wiki/wiki):

- Main Story

- Events

- Relationship Story

## 📔 Examples

```bash
./ba-search search -I -i "gamer"

# Output
01. Events/Alabaster Calling Card/02.html (Momoi): I'm a really nice person, and I'm not violent either! I'm just a regular, passionate gamer!
02. Events/New Year March/01.html (Maki): Gamers from every faction want to watch the sunrise, so there's an annual PVP event before dawn!
03. Events/Summer Sky's Promise/09.html (Nagisa): How...odd. Seia, are you some sort of...gamer?
04. Main Story/Volume 2/Chapter 2/Episode_02.html (Momoi): B-But I can't end on a loss. It hurts my pride as a gamer...
05. Main Story/Volume 2/Chapter 2/Episode_02.html (Yuzu): Yup. That's what a gamer does, after all.
06. Main Story/Volume 2/Chapter 2/Episode_05.html (Aris): Neru still has a long way to go as a gamer! She should be using combos like this...
07. Main Story/Volume 2/Chapter 2/Episode_19.html (INFO LINE): Next Episode: Gamers and Friends
08. Main Story/Volume EX/Chapter 2/Episode_23.html (Momoi): A-A waste?! Adding unnecessarily detailed real-life references is generally well-received...by some gamers!
09. Relationship Story/Midori/03.html (Midori): Gamers and collectors alike all have their eyes on the limited edition special bonus.
10. Relationship Story/Momoi/01.html (Momoi): That's a real blow to my pride as a gamer...
11. Relationship Story/Momoi/03.html (Momoi): As a gamer, I can't turn this opportunity down, Sensei!
12. Relationship Story/Momoi/03.html (Sensei): "Wait, as a gamer?"
13. Relationship Story/Momoi/05.html (Momoi): Fu. Fu. Fu... And "Operation: Make Midori a Gamer" was a resounding success.
14. Relationship Story/Momoi/05.html (Momoi): Huh? No. I was a gamer way before this!
15. Relationship Story/Momoi/05.html (Momoi): O-Of course! True immersion is a sign of a real gamer!
16. Relationship Story/Rei/01.html (Rei): What?! I-I'm not that amazing. I'm not even a gamer. I'm in baseball.
```

## 🚀 Getting Started

### Prerequisites

1. Blue Archive Wiki's local database, which can be found in this [Google Drive](https://drive.google.com/drive/folders/1OdtQNiUwygHA-05ZBxrlDaKdnf1selPV?usp=sharing).

2. `ba-search` requires **ripgrep** and **wget** to be installed on your system. The instructions to install both are below:

#### Windows (via Scoop)

```bash
scoop install ripgrep wget
```

#### Linux

```bash
# Ubuntu/Debian
sudo apt install ripgrep wget

# Fedora
sudo dnf install ripgrep wget

# Arch Linux
sudo pacman -S ripgrep wget
```

### Installation

You can download [latest release](https://github.com/AoToNatsu/ba-search/releases/tag/v1.0.0) for the binary.

*NOTICE: Mac users must manually compile the project.*

However, if you wish to compile this project from source, open your terminal, `cd` to the project directory, and run:

```bash
cargo build --release
```

## 💻 Usage

`ba-search` is split into two subcommands: `search` and `archive`.

### `search <INPUT>`

This will scrape the local wiki database. You should `cd` into the database directory beforehand.

If this subcommand crashes, it typically means `ripgrep` crashed. Otherwise, if no matches are found, `No matches for "[INPUT]"` will be displayed.

#### Search for dialogue with "Minori"

```bash
./ba-search search --input "Minori"
```

#### Search for dialogue with "Extremely"

```bash
./ba-search search --input "Extremely"
```

*Returns 2 matches as of 2026-06-05.*

#### Search for dialogue with "Extremely" (case insensitive)

```bash
./ba-search search --ignore-case --input "Extremely"
```

*Returns 40 matches as of 2026-06-05.*

#### `-c, --count`

Returns the number of matches.

```bash
./ba-search search --input "extremely" -c # Output: 37
```

#### `-w, --word-regexp`

Include only exact matches to the phrase; an input of "**may**" will never retrieve "**may**day," for example.

#### `-o, --outline`

Outline speaker and matches with asterisks.

No outline: `01. Main Story/Volume 2/Chapter 2/Episode_01.html (Himari): The artifact worshiped by the nameless priests...`

Outline: `01. Main Story/Volume 2/Chapter 2/Episode_01.html (**Himari**): The artifact worshiped by the **nameless priests**...`

#### `--student, --sensei, --info, --description`

Ignores the corresponding line types. `--sensei` ignores lines said by Sensei, etc.

### `archive <LINK> <END>`

This is a wrapper around `wget` to mass download pages.

The `archive` subcommand will list the link, range, and increment. You can omit this with the `-q` or `--quiet` flags.

The format string is `{}`, so you can insert that wherever the number changes.

#### Archive Natsu Relationship Story Ep 1-4

```bash
./ba-search archive -e 4 --link "https://bluearchive.wiki/wiki/Natsu/Relationship_Story/{}"
```

```bash
# OUTPUT; DO NOT PASTE INTO THE TERMINAL
Details:
Link: "https://bluearchive.wiki/wiki/Natsu/Relationship_Story/{}"
Range: 1-4
Increment: 1
```

Explanation: This runs `wget` for `https://bluearchive.wiki/wiki/Natsu/Relationship_Story/{}` where {} is `1`, `2`, `3`, and `4`; `wget` is run four times.

*NOTICE: The default start is 1. You can change this with the `-s` or `--start` flags, and you can change the increment with the `-i` or `--increment` flags.*

## Limitations

1. The `search` subcommand can only search dialogue, choice, info, and description lines. If you wish to filter further, you can pipe the output to `ripgrep` like so:

#### Only include "nameless priest" if Himari is the speaker

```bash
ba-search search --ignore-case --input "nameless priest" | rg "(Himari)"
```

#### Exclude matches of "nameless priest" if Himari is the speaker

```bash
ba-search search --ignore-case --input "nameless priest" | rg -v "(Himari)"
```

2. MomoTalk support is not implemented yet; it will be in the future.

3. Using `archive` *will not* automatically rename the files. Every file in the downloadable local wiki database was manually renamed to have leading zeros and the .html extension.
