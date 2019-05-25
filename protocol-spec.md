# Borker Protocol Specification

## Overview
A user can post a bork by submitting a transaction with an OP_RETURN output containing a special prefix, and their message.
The user is defined as the origin address of the first output spent by a transaction following this protocol.

A user's address must be P2PKH in order to enable an optimization in which we can determine the origin address without
looking up the referenced output.


## Message Types

### Identifying

#### Nickname
`Version (2 bytes)` `00` `<utf-8 encoded nickname> (0-77 bytes)`

#### Biography
`Version (2 bytes)` `01` `<biography> (0-77 bytes)`

#### Avatar
`Version (2 bytes)` `02` `<link to avatar image> (0-77 bytes)`

### Borking

#### Standard bork
`Version (2 bytes)` `03` `Nonce (1 byte)` `<message> (0-76 bytes)`

#### Comment
`Version (2 bytes)` `04` `Nonce (1 byte)` `Reference Nonce (1 byte)` `<message> (0-75 bytes)`

A comment references a previous bork, by checking the address of the first P2PKH, non-change, output,
and references the most recent bork posted by that user with a nonce equal to the provided reference nonce.

#### Legacy Comment
`Version (2 bytes)` `05` `Nonce (1 byte)` `Skip [VarInt] (1-9 bytes)` `Reference Nonce (1 byte)` `<message> (0-74 bytes)`

A legacy comment references a previous bork that is at least 256 messages old. It includes a varint indicating how many messages with the provided reference nonce to skip, going backwards.

#### Rebork
`Version (2 bytes)` `06` `Nonce (1 byte)` `Reference Nonce (1 byte)` `<message> (0-75 bytes)`

A rebork references a previous bork, by checking the address of the first P2PKH output,
and references the most recent bork posted by that user with a nonce equal to the provided reference nonce. Includes an optional message.

#### Legacy Rebork
`Version (2 bytes)` `07` `Nonce (1 byte)` `Skip [VarInt] (1-9 bytes)` `Reference Nonce (1 byte)` `<message> (0-74 bytes)`

A legacy like references a previous bork that is at least 256 messages old. It includes a varint indicating how many messages with the provided reference nonce to skip, going backwards.

#### Extension
`Version (2 bytes)` `08` `Nonce (1 byte)` `Reference Nonce (1 byte)` `<message> (0-75 bytes)`

This message type continues a previous message of types bork, comment, legacy comment, rebork, and legacy rebork.

#### Deleting a bork

`Version (2 bytes)` `09` `<txid to delete> (64 bytes)`

### Liking/Flagging

#### Like
`Version (2 bytes)` `0A` `Nonce (1 byte)` `Reference Nonce (1 byte)`

A like references a previous bork, by checking the address of the first P2PKH output,
and references the most recent bork posted by that user with a nonce equal to the provided reference nonce.

#### Legacy Like
`Version (2 bytes)` `0B` `Nonce (1 byte)` `Skip [VarInt] (1-9 bytes)` `Reference Nonce (1 byte)`

A legacy like references a previous bork that is at least 256 messages old. It includes a varint indicating how many messages with the provided reference nonce to skip, going backwards.

#### Flag

`Version (2 bytes)` `0C` `<txid to flag> (64 bytes)`

A flag marks a bork as inappropriate.

#### Unflag
`Version (2 bytes)` `0D` `<txid to unflag> (64 bytes)`

### Following/Blocking

#### Follow
`Version (2 bytes)` `0E` `<address to follow> (25 bytes)`

#### Unfollow
`Version (2 bytes)` `0F` `<address to unfollow> (25 bytes)`

#### Block
`Version (2 bytes)` `0G` `<address to block> (25 bytes)`

Blocking a user prevents them from viewing your profile and associated info, as well as viewing or interacting with your borks, comments, reborks, and extensions.

#### Unblock
`Version (2 bytes)` `0H` `<address to unblock> (25 bytes)`
