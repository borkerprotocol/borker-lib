# Borker Protocol Specification

## Overview
A user can post a bork by submitting a transaction with an OP_RETURN output containing a special prefix, and their message.
The user is defined as the origin address of the first output spent by a transaction following this protocol.

A user's address must be P2PKH in order to enable an optimization in which we can determine the origin address without
looking up the referenced output.


## Message Types

### Nickname Declaration
`Version (2 bytes)` `00` `<utf-8 encoded nickname> (0-77 bytes)`

### Biography Declaration
`Version (2 bytes)` `01` `<biography> (0-77 bytes)`

### Avatar Declaration
`Version (2 bytes)` `02` `<link to avatar image> (0-77 bytes)`

### Borks

#### Standard bork
`Version (2 bytes)` `03` `Nonce (1 byte)` `<message> (0-76 bytes)`

#### Reply
`Version (2 bytes)` `04` `Nonce (1 byte)` `Reference Nonce (1 bytes)` `<message> (0-75 bytes)`
A reply references a previous bork, by checking the address of the first non-OP_RETURN output,
and references the most recent bork posted by that user with a nonce equal to the provided reference nonce.

By convention, a reply to your most recent bork is seen as a continuation of it.

### Follows/Likes

#### Follow
`Version (2 bytes)` `05` `<address to follow> (26-34 bytes)`

#### Unfollow
`Version (2 bytes)` `06` `<address to unfollow> (26-34 bytes)`

#### Like
`Version (2 bytes)` `07` `Reference Nonce (1 byte)`
A like references a previous bork, by checking the address of the first non-OP_RETURN output,
and references the most recent bork posted by that user with a nonce equal to the provided reference nonce.

#### Rebork
`Version (2 bytes)` `08` `Reference Nonce (1 byte)`
A rebork references a previous bork, by checking the address of the first non-OP_RETURN output,
and references the most recent bork posted by that user with a nonce equal to the provided reference nonce.
