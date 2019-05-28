# Borker Protocol Specification

## Overview
A user can post a bork by submitting a transaction with an OP_RETURN output containing a special prefix, and their message.
The user is defined as the origin address of the first output spent by a transaction following this protocol.

A user's address must be P2PKH in order to enable an optimization in which we can determine the origin address without
looking up the referenced output.


## Message Types

### Identifying

#### Nickname
`Version (2 bytes)` `00` `<nickname> (0-77 bytes)`

#### Biography
`Version (2 bytes)` `01` `<biography> (0-77 bytes)`

#### Avatar
`Version (2 bytes)` `02` `<link to avatar image> (0-77 bytes)`

### Borking

#### Standard bork
`Version (2 bytes)` `03` `Nonce (1 byte)` `<message> (0-76 bytes)`

#### Comment
`Version (2 bytes)` `04` `Nonce (1 byte)` `Ref (VarString 2-33 bytes)` `<message> (0-74 bytes)`

A comment references a previous bork, by checking the address of the first P2PKH, non-change, output,
and references the most recent bork posted by that user with a txid with the same prefix as the provided `Ref` string.

#### Rebork
`Version (2 bytes)` `05` `Nonce (1 byte)` `Ref (VarString 2-33 bytes)` `<message> (0-74 bytes)`

A rebork references a previous bork, by checking the address of the first P2PKH output,
and references the most recent bork posted by that user with a txid with the same prefix as the provided `Ref` string. Includes an optional message.

#### Extension
`Version (2 bytes)` `06` `Nonce (1 byte)` `Index (1 byte)` `<message> (0-75 bytes)`

This message type continues a previous message of types bork, comment, and rebork. It is a continuation of the most recent message with the same nonce. Index must start at 1.

#### Deleting a bork

`Version (2 bytes)` `07` `Ref (VarString 2-33 bytes)`

Deletes a previous message, referred to by the most recent bork from the same address with a txid with the same prefix as the provided `Ref` string.

### Liking/Flagging

#### Like
`Version (2 bytes)` `08` `Ref (VarString 2-33 bytes)`

A like references a previous bork, by checking the address of the first P2PKH output,
and references the most recent bork posted by that user with the most recent txid with the same prefix as the provided `Ref` string.

#### Unlike
`Version (2 bytes)` `09` `<txid to unlike> (32 bytes)`

Removes a like from a previous bork, by txid.

#### Flag

`Version (2 bytes)` `0A` `<txid to flag> (32 bytes)`

A flag marks a bork as inappropriate.

#### Unflag
`Version (2 bytes)` `0B` `<txid to unflag> (32 bytes)`

Removes a flag from a previous bork, by txid.

### Following/Blocking

#### Follow
`Version (2 bytes)` `0C` `<pubkey hash to follow> (20 bytes)`

#### Unfollow
`Version (2 bytes)` `0D` `<pubkey hash to unfollow> (20 bytes)`

#### Block
`Version (2 bytes)` `0E` `<pubkey hash to block> (20 bytes)`

Blocking a user prevents them from viewing your profile and associated info, as well as viewing or interacting with your borks, comments, reborks, and extensions. (This is only enforced client side).

#### Unblock
`Version (2 bytes)` `0F` `<pubkey hash to unblock> (20 bytes)`
