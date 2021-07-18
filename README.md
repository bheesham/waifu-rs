waifu-rs
========

`waifu-rs` is a bot for SaltyBet which will:

* make a record of the outcome matches;
* calculate the elo of players for those matches;
* place bets based on the elo of players.

Your milage may vary. I've been stuck in the mines, though there are streaks
where the bot will get into the millions of fake internet money.

# Running

You'll need the following variables set in your environment:

```fish
# Your SaltyBet credentials
set -x SB_USERNAME ""
set -x SB_PASSWORD ""

# The path of the sqlite3 database to save state to.
set -x W_FILE_PATH prod.db

# The SaltyBet "API" endpoints. These can probably be hard-coded, but eh.
set -x SB_BET_URL "https://www.saltybet.com/ajax_place_bet.php"
set -x SB_LOGIN_URL "https://www.saltybet.com/authenticate?signin=1"
set -x SB_REFERER_URL "https://www.saltybet.com/"
set -x SB_INDEX "https://www.saltybet.com/"

# Aaaand logging settings.
set -x RUST_LOG "waifu=info"
```

*n.b.*: the shell syntax is for fish. You'll need to use whatever syntax is
compatible with your shell.

With those variables set, you can run the command:

```
cargo run --release
```

Happy betting!

## Seeding the database

I added the data I collected into `contrib/data.sql`. You can seed your bot by
using this data. To do that you'll need to run:

```
sqlite3 prod.db
.read contrib/data.sql
```

Once you've seeded your database be sure to update the `W_FILE_PATH`
environment variable.

# History

This is the third incarnation of this bot. The first was a JS snippet which
would randomly bet on red or blue. The second was a bot written in Clojure and,
using elo as a heuristic for placing bets. This project is the third, and
probably final for now, series in the saga.

From running the bot for a while I've noticed there are highs and lows because
of the betting strategy we use. In particular, we *always* place a bet if we
can. In reality, we should really avoid placing a bet if we're not certain of
the outcome. The amount of money we place on a bet should probably also change,
since we'll always try to be 10% of our current balance.

If you care enough you can probably write a bot which performs much better. I
used this project mostly as a way to learn a bit of Rust.

# License

```
waifu-rs: a bot to place bets on SaltyBet.
Copyright (C) 2021  Bheesham Persaud

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
```
