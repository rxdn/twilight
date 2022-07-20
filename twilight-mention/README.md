# twilight-mention

[![codecov badge][]][codecov link] [![discord badge][]][discord link] [![github badge][]][github link] [![license badge][]][license link] ![rust badge]

`twilight-mention` is a utility crate for the Discord [`twilight-rs`]
ecosystem to mention its model types and parse those mentions.

With this library, you can create mentions for various types, such as users,
emojis, roles, members, or channels.

## Examples

Create a mention formatter for a user ID, and then format it in a message:

```rust
use twilight_mention::Mention;
use twilight_model::id::{marker::UserMarker, Id};

let user_id = Id::<UserMarker>::new(123);
let message = format!("Hey there, {}!", user_id.mention());
```

[`twilight-rs`]: https://github.com/twilight-rs/twilight
[codecov badge]: https://img.shields.io/codecov/c/gh/twilight-rs/twilight?logo=codecov&style=for-the-badge&token=E9ERLJL0L2
[codecov link]: https://app.codecov.io/gh/twilight-rs/twilight/
[discord badge]: https://img.shields.io/discord/745809834183753828?color=%237289DA&label=discord%20server&logo=discord&style=for-the-badge
[discord link]: https://discord.gg/7jj8n7D
[github badge]: https://img.shields.io/badge/github-twilight-6f42c1.svg?style=for-the-badge&logo=github
[github link]: https://github.com/twilight-rs/twilight
[license badge]: https://img.shields.io/badge/license-ISC-blue.svg?style=for-the-badge&logo=pastebin
[license link]: https://github.com/twilight-rs/twilight/blob/main/LICENSE.md
[rust badge]: https://img.shields.io/badge/rust-1.60+-93450a.svg?style=for-the-badge&logo=rust
