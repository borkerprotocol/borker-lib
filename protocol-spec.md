# Borker Protocol Specification

## Overview
A user can post a bork by submitting a transaction with an OP_RETURN output containing a special prefix, and their message.
The user is defined as the origin address of the first output spent by a transaction following this protocol.

A user's address must be P2PKH in order to enable an optimization in which we can determine the origin address without
looking up the referenced output.


## Bork Types

### Nickname
`Version (2 bytes)` `00` `<nickname> (0-77 bytes)`

Set your nickname. Cannot be extended.

### Biography
`Version (2 bytes)` `01` `<biography> (0-77 bytes)`

Set your biography. Cannot be extended.

### Avatar
`Version (2 bytes)` `02` `<link to avatar image> (0-77 bytes)`

Set your avatar by providing a link to the image. Cannot be extended.

### Post
`Version (2 bytes)` `03` `Nonce (1 byte)` `<message> (0-76 bytes)`

Bork it loud.

### Comment
`Version (2 bytes)` `04` `Nonce (1 byte)` `Ref (VarString 2-33 bytes)` `<message> (0-74 bytes)`

A comment references a previous bork, by checking the address of the first P2PKH, non-change, output,
and references the most recent bork posted by that user with a txid with the same prefix as the provided `Ref` string.

### Rebork
`Version (2 bytes)` `05` `Nonce (1 byte)` `Ref (VarString 2-33 bytes)` `<message> (0-74 bytes)`

A rebork references a previous bork, by checking the address of the first P2PKH output,
and references the most recent bork posted by that user with a txid with the same prefix as the provided `Ref` string. Includes an optional message.

### Extension
`Version (2 bytes)` `06` `Nonce (1 byte)` `Index (1 byte)` `<message> (0-75 bytes)`

This bork type continues a previous bork of type Post, Comment, or Rebork. It is a continuation of the most recent bork with the same nonce. Index must start at 1 since the original bork already has index 0.

### Like
`Version (2 bytes)` `07` `Ref (VarString 2-33 bytes)`

A like references a previous bork, by checking the address of the first P2PKH output,
and references the most recent bork posted by that user with the most recent txid with the same prefix as the provided `Ref` string.

### Flag

`Version (2 bytes)` `08` `<txid to flag> (32 bytes)`

A flag marks a bork as inappropriate.

### Follow
`Version (2 bytes)` `09` `<address to follow> (20 bytes)`

### Block
`Version (2 bytes)` `0A` `<address to block> (20 bytes)`

Blocking a user prevents them from viewing your profile and associated info, as well as viewing or interacting with your borks.

### Delete

`Version (2 bytes)` `0B` `Ref (VarString 2-33 bytes)`

Deletes a previous bork, referred to by the most recent non-deleted bork from the same address with a txid with the same prefix as the provided `Ref` string. Use use this bork type to Unlike, Unflag, Unfollow, and Unblock. Delete type has no effect on previous borks of type Nickname, Biography, Avatar, or Delete.

*Delete only removes borks from borker-server. The original bork still exists on the underlying blockchain and could therefore be recovered.*
