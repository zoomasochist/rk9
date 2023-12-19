<p align="center">
  <a href="https://xkcd.com/1128/">
    <img src="https://imgs.xkcd.com/comics/fifty_shades.png">
  </a>
</p>

# /rk9/

A Discord bot for (mostly furry) gooners.

## Running

```shell
$ cat >rk9.toml <<EOF
discord_token = "<your bot token>"
database_path = "./rk9.sqlite"
accent_colour = 0xb4c4f9
EOF
$ cargo build --release
$ target/release/rk9
```